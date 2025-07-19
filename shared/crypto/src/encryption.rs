use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;
use aes_gcm::aead::rand_core::RngCore;
//use std::io::{Read, Write};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub data: Vec<u8>,
    pub nonce: Vec<u8>,
    pub tag: Vec<u8>,
}

#[derive(Clone)]
pub struct EncryptionKey {
    key: [u8; 32],
}

impl EncryptionKey {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }
    
    pub fn generate() -> Self {
        let mut key = [0u8; 32];
        let mut rng = OsRng;
        rng.fill_bytes(&mut key);
        Self { key }
    }
    
    pub fn from_slice(slice: &[u8]) -> Result<Self, CryptoError> {
        if slice.len() != 32 {
            return Err(CryptoError::InvalidKeySize);
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(slice);
        Ok(Self { key })
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        &self.key
    }
}

impl Drop for EncryptionKey {
    fn drop(&mut self) {
        self.key.zeroize();
    }
}

impl Zeroize for EncryptionKey {
    fn zeroize(&mut self) {
        self.key.zeroize();
    }
}

pub struct FileEncryption {
    cipher: Aes256Gcm,
}

impl FileEncryption {
    pub fn new(key: &EncryptionKey) -> Self {
        let aes_key = Key::<Aes256Gcm>::from_slice(key.as_bytes());
        let cipher = Aes256Gcm::new(aes_key);
        Self { cipher }
    }
    
    pub fn encrypt(&self, data: &[u8]) -> Result<EncryptedData, CryptoError> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self.cipher
            .encrypt(&nonce, data)
            .map_err(|_| CryptoError::EncryptionFailed)?;
        
        // Extract the tag (last 16 bytes for GCM)
        let tag_start = ciphertext.len().saturating_sub(16);
        let (encrypted_data, tag) = ciphertext.split_at(tag_start);
        
        Ok(EncryptedData {
            data: encrypted_data.to_vec(),
            nonce: nonce.to_vec(),
            tag: tag.to_vec(),
        })
    }
    
    pub fn decrypt(&self, encrypted: &EncryptedData) -> Result<Vec<u8>, CryptoError> {
        if encrypted.nonce.len() != 12 {
            return Err(CryptoError::InvalidNonce);
        }
        
        let nonce = Nonce::from_slice(&encrypted.nonce);
        
        // Reconstruct the ciphertext with tag
        let mut ciphertext = encrypted.data.clone();
        ciphertext.extend_from_slice(&encrypted.tag);
        
        self.cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| CryptoError::DecryptionFailed)
    }
    
    pub fn encrypt_stream<R: std::io::Read, W: std::io::Write>(
        &self,
        reader: &mut R,
        writer: &mut W,
        chunk_size: usize,
    ) -> Result<(), CryptoError> {
        let mut buffer = vec![0u8; chunk_size];
        
        loop {
            let bytes_read = reader.read(&mut buffer)
                .map_err(|_| CryptoError::IoError)?;
            
            if bytes_read == 0 {
                break;
            }
            
            let chunk = &buffer[..bytes_read];
            let encrypted = self.encrypt(chunk)?;
            
            // Write encrypted chunk with length prefix
            let chunk_len = (encrypted.data.len() + encrypted.nonce.len() + encrypted.tag.len()) as u32;
            writer.write_all(&chunk_len.to_le_bytes())
                .map_err(|_| CryptoError::IoError)?;
            writer.write_all(&encrypted.nonce)
                .map_err(|_| CryptoError::IoError)?;
            writer.write_all(&encrypted.tag)
                .map_err(|_| CryptoError::IoError)?;
            writer.write_all(&encrypted.data)
                .map_err(|_| CryptoError::IoError)?;
        }
        
        Ok(())
    }
    
    pub fn decrypt_stream<R: std::io::Read, W: std::io::Write>(
        &self,
        reader: &mut R,
        writer: &mut W,
    ) -> Result<(), CryptoError> {
        let mut len_buffer = [0u8; 4];
        
        loop {
            match reader.read_exact(&mut len_buffer) {
                Ok(_) => {},
                Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(_) => return Err(CryptoError::IoError),
            }
            
            let chunk_len = u32::from_le_bytes(len_buffer) as usize;
            let mut chunk_buffer = vec![0u8; chunk_len];
            
            reader.read_exact(&mut chunk_buffer)
                .map_err(|_| CryptoError::IoError)?;
            
            if chunk_len < 28 { // 12 (nonce) + 16 (tag) minimum
                return Err(CryptoError::InvalidChunk);
            }
            
            let encrypted = EncryptedData {
                nonce: chunk_buffer[0..12].to_vec(),
                tag: chunk_buffer[12..28].to_vec(),
                data: chunk_buffer[28..].to_vec(),
            };
            
            let decrypted = self.decrypt(&encrypted)?;
            writer.write_all(&decrypted)
                .map_err(|_| CryptoError::IoError)?;
        }
        
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Invalid key size")]
    InvalidKeySize,
    #[error("Invalid nonce")]
    InvalidNonce,
    #[error("Encryption failed")]
    EncryptionFailed,
    #[error("Decryption failed")]
    DecryptionFailed,
    #[error("IO error")]
    IoError,
    #[error("Invalid chunk")]
    InvalidChunk,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encrypt_decrypt() {
        let key = EncryptionKey::generate();
        let encryption = FileEncryption::new(&key);
        
        let data = b"Hello, World!";
        let encrypted = encryption.encrypt(data).unwrap();
        let decrypted = encryption.decrypt(&encrypted).unwrap();
        
        assert_eq!(data, decrypted.as_slice());
    }
    
    #[test]
    fn test_stream_encryption() {
        let key = EncryptionKey::generate();
        let encryption = FileEncryption::new(&key);
        
        let data = b"This is a longer piece of data that will be encrypted in chunks";
        let mut reader = std::io::Cursor::new(data);
        let mut encrypted_buffer = Vec::new();
        
        encryption.encrypt_stream(&mut reader, &mut encrypted_buffer, 16).unwrap();
        
        let mut encrypted_reader = std::io::Cursor::new(encrypted_buffer);
        let mut decrypted_buffer = Vec::new();
        
        encryption.decrypt_stream(&mut encrypted_reader, &mut decrypted_buffer).unwrap();
        
        assert_eq!(data, decrypted_buffer.as_slice());
    }
}
