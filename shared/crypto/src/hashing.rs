use blake3::Hasher;
use std::io::{Read, BufReader};
use std::fs::File;
use std::path::Path;

pub struct FileHasher {
    hasher: Hasher,
}

impl FileHasher {
    pub fn new() -> Self {
        Self {
            hasher: Hasher::new(),
        }
    }
    
    pub fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }
    
    pub fn finalize(self) -> String {
        self.hasher.finalize().to_hex().to_string()
    }
    
    pub fn hash_bytes(data: &[u8]) -> String {
        blake3::hash(data).to_hex().to_string()
    }
    
    pub fn hash_file<P: AsRef<Path>>(path: P) -> Result<String, std::io::Error> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut hasher = Hasher::new();
        
        let mut buffer = [0u8; 8192];
        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        
        Ok(hasher.finalize().to_hex().to_string())
    }
    
    pub fn hash_stream<R: Read>(reader: &mut R) -> Result<String, std::io::Error> {
        let mut hasher = Hasher::new();
        let mut buffer = [0u8; 8192];
        
        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        
        Ok(hasher.finalize().to_hex().to_string())
    }
    
    // For block-level hashing (useful for sync)
    pub fn hash_blocks<R: Read>(
        reader: &mut R, 
        block_size: usize
    ) -> Result<Vec<BlockHash>, std::io::Error> {
        let mut blocks = Vec::new();
        let mut buffer = vec![0u8; block_size];
        let mut offset = 0;
        
        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            
            let block_data = &buffer[..bytes_read];
            let hash = Self::hash_bytes(block_data);
            
            blocks.push(BlockHash {
                offset,
                size: bytes_read,
                hash,
            });
            
            offset += bytes_read;
        }
        
        Ok(blocks)
    }
}

impl Default for FileHasher {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockHash {
    pub offset: usize,
    pub size: usize,
    pub hash: String,
}

// Rolling hash for efficient diff detection (similar to rsync)
pub struct RollingHasher {
    window_size: usize,
    a: u32,
    b: u32,
    window: Vec<u8>,
    position: usize,
}

impl RollingHasher {
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            a: 0,
            b: 0,
            window: vec![0; window_size],
            position: 0,
        }
    }
    
    pub fn update(&mut self, byte: u8) -> u32 {
        let old_byte = self.window[self.position];
        self.window[self.position] = byte;
        self.position = (self.position + 1) % self.window_size;
        
        // Update rolling hash values
        self.a = self.a.wrapping_sub(old_byte as u32).wrapping_add(byte as u32);
        self.b = self.b.wrapping_sub((self.window_size as u32) * (old_byte as u32))
            .wrapping_add(self.a);
        
        (self.a & 0xFFFF) | ((self.b & 0xFFFF) << 16)
    }
    
    pub fn hash(&self) -> u32 {
        (self.a & 0xFFFF) | ((self.b & 0xFFFF) << 16)
    }
    
    pub fn find_matches<R: Read>(
        &mut self, 
        reader: &mut R, 
        target_hashes: &[u32]
    ) -> Result<Vec<Match>, std::io::Error> {
        let mut matches = Vec::new();
        let mut buffer = [0u8; 1];
        let mut position = 0;
        
        while reader.read_exact(&mut buffer).is_ok() {
            let hash = self.update(buffer[0]);
            
            if target_hashes.contains(&hash) {
                matches.push(Match {
                    position,
                    hash,
                    length: self.window_size,
                });
            }
            
            position += 1;
        }
        
        Ok(matches)
    }
}

#[derive(Debug, Clone)]
pub struct Match {
    pub position: usize,
    pub hash: u32,
    pub length: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    
    #[test]
    fn test_hash_bytes() {
        let data = b"Hello, World!";
        let hash1 = FileHasher::hash_bytes(data);
        let hash2 = FileHasher::hash_bytes(data);
        
        assert_eq!(hash1, hash2);
        assert!(!hash1.is_empty());
    }
    
    #[test]
    fn test_hash_stream() {
        let data = b"This is test data for streaming hash";
        let mut cursor = Cursor::new(data);
        let hash = FileHasher::hash_stream(&mut cursor).unwrap();
        
        assert!(!hash.is_empty());
        assert_eq!(hash, FileHasher::hash_bytes(data));
    }
    
    #[test]
    fn test_rolling_hasher() {
        let mut hasher = RollingHasher::new(4);
        let data = b"abcdefgh";
        
        let mut hashes = Vec::new();
        for &byte in data {
            let hash = hasher.update(byte);
            hashes.push(hash);
        }
        
        assert_eq!(hashes.len(), data.len());
        // Rolling hash should produce consistent results for same windows
    }
    
    #[test]
    fn test_block_hashing() {
        let data = b"This is a longer piece of data that will be split into blocks";
        let mut cursor = Cursor::new(data);
        let blocks = FileHasher::hash_blocks(&mut cursor, 16).unwrap();
        
        assert!(!blocks.is_empty());
        assert_eq!(blocks[0].offset, 0);
        assert!(blocks[0].size <= 16);
    }
}
