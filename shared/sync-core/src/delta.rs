use crate::{SyncError, SyncOptions};
use fileshare_crypto::{BlockHash, FileHasher, RollingHasher};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDelta {
    pub file_id: String,
    pub source_checksum: String,
    pub target_checksum: String,
    pub operations: Vec<DeltaOperation>,
    pub compressed_size: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeltaOperation {
    Copy {
        source_offset: u64,
        target_offset: u64,
        length: u64,
    },
    Insert {
        target_offset: u64,
        data: Vec<u8>,
    },
    Delete {
        source_offset: u64,
        length: u64,
    },
}

pub struct DeltaGenerator {
    options: SyncOptions,
    rolling_hasher: RollingHasher,
}

impl DeltaGenerator {
    pub fn new(options: SyncOptions) -> Self {
        let rolling_hasher = RollingHasher::new(options.chunk_size.min(4096));
        Self {
            options,
            rolling_hasher,
        }
    }

    /// Generate delta between two files
    pub fn generate_delta<R1, R2>(
        &mut self,
        mut source: R1,
        mut target: R2,
        source_blocks: &[BlockHash],
    ) -> Result<FileDelta, SyncError>
    where
        R1: Read + Seek,
        R2: Read + Seek,
    {
        // Calculate checksums
        source.seek(SeekFrom::Start(0))?;
        target.seek(SeekFrom::Start(0))?;

        let source_checksum = FileHasher::hash_stream(&mut source)?;
        let target_checksum = FileHasher::hash_stream(&mut target)?;

        if source_checksum == target_checksum {
            // Files are identical
            return Ok(FileDelta {
                file_id: String::new(),
                source_checksum,
                target_checksum,
                operations: vec![],
                compressed_size: None,
            });
        }

        // Reset streams
        source.seek(SeekFrom::Start(0))?;
        target.seek(SeekFrom::Start(0))?;

        // Generate block map from source
        let block_map = Self::create_block_map(source_blocks);

        // Find matching blocks in target
        let operations = self.find_delta_operations(&mut target, &block_map)?;

        let mut delta = FileDelta {
            file_id: String::new(),
            source_checksum,
            target_checksum,
            operations,
            compressed_size: None,
        };

        // Compress delta if enabled
        if self.options.compression_enabled {
            delta = self.compress_delta(delta)?;
        }

        Ok(delta)
    }

    fn create_block_map(blocks: &[BlockHash]) -> HashMap<String, Vec<&BlockHash>> {
        let mut map = HashMap::new();
        for block in blocks {
            map.entry(block.hash.clone())
                .or_insert_with(Vec::new)
                .push(block);
        }
        map
    }

    fn find_delta_operations<R: Read>(
        &mut self,
        target: &mut R,
        block_map: &HashMap<String, Vec<&BlockHash>>,
    ) -> Result<Vec<DeltaOperation>, SyncError> {
        let mut operations = Vec::new();
        let mut buffer = vec![0u8; self.options.chunk_size];
        let mut target_offset = 0u64;
        let mut pending_insert = Vec::new();

        loop {
            let bytes_read = target.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let chunk = &buffer[..bytes_read];
            let chunk_hash = FileHasher::hash_bytes(chunk);

            if let Some(source_blocks) = block_map.get(&chunk_hash) {
                // Found matching block
                if !pending_insert.is_empty() {
                    // Flush pending insert
                    operations.push(DeltaOperation::Insert {
                        target_offset: target_offset - pending_insert.len() as u64,
                        data: pending_insert.clone(),
                    });
                    pending_insert.clear();
                }

                // Use first matching block (could be optimized)
                let source_block = source_blocks[0];
                operations.push(DeltaOperation::Copy {
                    source_offset: source_block.offset as u64,
                    target_offset,
                    length: bytes_read as u64,
                });
            } else {
                // No match, add to pending insert
                pending_insert.extend_from_slice(chunk);
            }

            target_offset += bytes_read as u64;
        }

        // Flush any remaining pending insert
        if !pending_insert.is_empty() {
            operations.push(DeltaOperation::Insert {
                target_offset: target_offset - pending_insert.len() as u64,
                data: pending_insert,
            });
        }

        Ok(operations)
    }

    fn compress_delta(&self, mut delta: FileDelta) -> Result<FileDelta, SyncError> {
        let serialized = serde_json::to_vec(&delta.operations).map_err(SyncError::Serialization)?;

        let compressed = zstd::bulk::compress(&serialized, 3)
            .map_err(|e| SyncError::Compression(e.to_string()))?;

        if compressed.len() < serialized.len() {
            delta.compressed_size = Some(compressed.len());
            // In a real implementation, you'd store the compressed data
            // For now, we just record that compression was beneficial
        }

        Ok(delta)
    }
}

pub struct DeltaApplier {
    options: SyncOptions,
}

impl DeltaApplier {
    pub fn new(options: SyncOptions) -> Self {
        Self { options }
    }

    /// Apply delta to reconstruct target file
    pub fn apply_delta<R, W>(
        &self,
        mut source: R,
        mut target: W,
        delta: &FileDelta,
    ) -> Result<(), SyncError>
    where
        R: Read + Seek,
        W: std::io::Write,
    {
        for operation in &delta.operations {
            match operation {
                DeltaOperation::Copy {
                    source_offset,
                    length,
                    ..
                } => {
                    source.seek(SeekFrom::Start(*source_offset))?;
                    let mut buffer = vec![0u8; *length as usize];
                    source.read_exact(&mut buffer)?;
                    target.write_all(&buffer)?;
                }
                DeltaOperation::Insert { data, .. } => {
                    target.write_all(data)?;
                }
                DeltaOperation::Delete { .. } => {
                    // Delete operations are implicit when reconstructing
                    // (we don't copy the deleted regions)
                }
            }
        }

        Ok(())
    }

    /// Verify delta application by checking checksums
    pub fn verify_delta_application<R>(
        &self,
        mut result: R,
        expected_checksum: &str,
    ) -> Result<bool, SyncError>
    where
        R: Read,
    {
        let actual_checksum = FileHasher::hash_stream(&mut result)?;
        Ok(actual_checksum == expected_checksum)
    }
}

/// Optimized delta for small changes
pub struct SmallFileDelta {
    pub changes: Vec<ByteChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByteChange {
    pub offset: u64,
    pub old_bytes: Vec<u8>,
    pub new_bytes: Vec<u8>,
}

impl SmallFileDelta {
    pub fn generate<R1, R2>(mut source: R1, mut target: R2) -> Result<Self, SyncError>
    where
        R1: Read,
        R2: Read,
    {
        let mut source_data = Vec::new();
        let mut target_data = Vec::new();

        source.read_to_end(&mut source_data)?;
        target.read_to_end(&mut target_data)?;

        let changes = Self::diff_bytes(&source_data, &target_data);
        Ok(Self { changes })
    }

    fn diff_bytes(source: &[u8], target: &[u8]) -> Vec<ByteChange> {
        let mut changes = Vec::new();
        let mut i = 0;
        let mut j = 0;

        while i < source.len() || j < target.len() {
            if i < source.len() && j < target.len() && source[i] == target[j] {
                // Bytes match, continue
                i += 1;
                j += 1;
                continue;
            }

            // Find the extent of the difference
            let start_i = i;
            let start_j = j;

            // Simple diff: find next matching position
            while i < source.len() && j < target.len() {
                if source[i] == target[j] {
                    break;
                }
                i += 1;
                j += 1;
            }

            // If we hit the end of one or both, extend to the end
            if i == source.len() {
                j = target.len();
            } else if j == target.len() {
                i = source.len();
            }

            changes.push(ByteChange {
                offset: start_i as u64,
                old_bytes: source[start_i..i].to_vec(),
                new_bytes: target[start_j..j].to_vec(),
            });
        }

        changes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_delta_generation_identical_files() {
        let options = SyncOptions::default();
        let mut generator = DeltaGenerator::new(options);

        let data = b"Hello, World!";
        let source = Cursor::new(data);
        let target = Cursor::new(data);

        let source_blocks = vec![BlockHash {
            offset: 0,
            size: data.len(),
            hash: FileHasher::hash_bytes(data),
        }];

        let delta = generator
            .generate_delta(source, target, &source_blocks)
            .unwrap();
        assert_eq!(delta.source_checksum, delta.target_checksum);
        assert!(delta.operations.is_empty());
    }

    #[test]
    fn test_small_file_delta() {
        let source = b"Hello, World!";
        let target = b"Hello, Rust!";

        let delta = SmallFileDelta::generate(Cursor::new(source), Cursor::new(target)).unwrap();

        assert!(!delta.changes.is_empty());

        // Should detect the change from "World" to "Rust"
        let change = &delta.changes[0];
        assert!(change.old_bytes.contains(&b'W'));
        assert!(change.new_bytes.contains(&b'R'));
    }

    #[test]
    fn test_delta_application() {
        let options = SyncOptions::default();
        let applier = DeltaApplier::new(options);

        let source_data = b"Hello, World!";
        let insert_data = b" from Rust";

        let delta = FileDelta {
            file_id: "test".to_string(),
            source_checksum: "source".to_string(),
            target_checksum: "target".to_string(),
            operations: vec![
                DeltaOperation::Copy {
                    source_offset: 0,
                    target_offset: 0,
                    length: 7, // "Hello, "
                },
                DeltaOperation::Insert {
                    target_offset: 7,
                    data: insert_data.to_vec(),
                },
                DeltaOperation::Copy {
                    source_offset: 7,
                    target_offset: 17,
                    length: 6, // "World!"
                },
            ],
            compressed_size: None,
        };

        let source = Cursor::new(source_data);
        let mut result = Vec::new();

        applier.apply_delta(source, &mut result, &delta).unwrap();

        let expected = b"Hello,  from RustWorld!";
        assert_eq!(result, expected);
    }
}
