pub mod delta;
pub mod conflict;
pub mod merger;
pub mod engine;
pub mod chunker;

pub use delta::*;
// TODO: Uncomment when these modules are implemented
// pub use conflict::*;
// pub use merger::*;
// pub use engine::*;
// pub use chunker::*;

use protocol::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub path: String,
    pub hash: String,
    pub modified: DateTime<Utc>,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncDelta {
    pub added: Vec<SyncState>,
    pub modified: Vec<SyncState>,
    pub deleted: Vec<String>,
}

pub struct SyncEngine {
    local_state: HashMap<String, SyncState>,
}

impl SyncEngine {
    pub fn new() -> Self {
        Self {
            local_state: HashMap::new(),
        }
    }

    pub fn compute_delta(&self, remote_state: &HashMap<String, SyncState>) -> SyncDelta {
        let mut delta = SyncDelta {
            added: Vec::new(),
            modified: Vec::new(),
            deleted: Vec::new(),
        };

        // Find added and modified files
        for (path, remote_file) in remote_state {
            match self.local_state.get(path) {
                None => delta.added.push(remote_file.clone()),
                Some(local_file) => {
                    if local_file.hash != remote_file.hash {
                        delta.modified.push(remote_file.clone());
                    }
                }
            }
        }

        // Find deleted files
        for path in self.local_state.keys() {
            if !remote_state.contains_key(path) {
                delta.deleted.push(path.clone());
            }
        }

        delta
    }

    pub fn update_state(&mut self, path: String, state: SyncState) {
        self.local_state.insert(path, state);
    }

    pub fn remove_state(&mut self, path: &str) {
        self.local_state.remove(path);
    }
}

#[derive(Debug, Clone)]
pub struct SyncOptions {
    pub chunk_size: usize,
    pub compression_enabled: bool,
    pub conflict_resolution: ConflictResolutionMode,
    pub bandwidth_limit: Option<u64>, // bytes per second
    pub max_file_size: Option<u64>,
    pub exclude_patterns: Vec<String>,
}

impl Default for SyncOptions {
    fn default() -> Self {
        Self {
            chunk_size: 1024 * 1024, // 1MB
            compression_enabled: true,
            conflict_resolution: ConflictResolutionMode::Manual,
            bandwidth_limit: None,
            max_file_size: Some(10 * 1024 * 1024 * 1024), // 10GB
            exclude_patterns: vec![
                ".DS_Store".to_string(),
                "Thumbs.db".to_string(),
                ".tmp".to_string(),
                ".temp".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConflictResolutionMode {
    Manual,
    LocalWins,
    RemoteWins,
    Timestamp,
    Size,
}

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Conflict detected: {0}")]
    Conflict(String),
    #[error("File too large: {size} bytes")]
    FileTooLarge { size: u64 },
    #[error("Network error: {0}")]
    Network(String),
    #[error("Compression error: {0}")]
    Compression(String),
    #[error("Checksum mismatch")]
    ChecksumMismatch,
    #[error("Sync engine error: {0}")]
    Engine(String),
}
