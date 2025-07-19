pub mod encryption;
pub mod hashing;
pub mod password;
pub mod key_derivation;

pub use encryption::*;
pub use hashing::*;
pub use password::*;
pub use key_derivation::*;

use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit};
use blake3::Hasher;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoConfig {
    pub encryption_algorithm: String,
    pub key_size: usize,
    pub pbkdf2_iterations: u32,
    pub salt_size: usize,
}

impl Default for CryptoConfig {
    fn default() -> Self {
        Self {
            encryption_algorithm: "AES-256-GCM".to_string(),
            key_size: 32, // 256 bits
            pbkdf2_iterations: 100_000,
            salt_size: 16,
        }
    }
}

#[derive(Clone)]
pub struct CryptoEngine {
    cipher: Aes256Gcm,
}

impl CryptoEngine {
    pub fn new(key: &[u8; 32]) -> Self {
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        Self { cipher }
    }

    pub fn encrypt(&self, data: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>, String> {
        let nonce = Nonce::from_slice(nonce);
        self.cipher.encrypt(nonce, data)
            .map_err(|e| format!("Encryption failed: {}", e))
    }

    pub fn decrypt(&self, data: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>, String> {
        let nonce = Nonce::from_slice(nonce);
        self.cipher.decrypt(nonce, data)
            .map_err(|e| format!("Decryption failed: {}", e))
    }
}

pub fn hash_file(data: &[u8]) -> String {
    let mut hasher = Hasher::new();
    hasher.update(data);
    hasher.finalize().to_hex().to_string()
}

pub fn hash_password(password: &str, salt: &str) -> String {
    let mut hasher = Hasher::new();
    hasher.update(password.as_bytes());
    hasher.update(salt.as_bytes());
    hasher.finalize().to_hex().to_string()
}
