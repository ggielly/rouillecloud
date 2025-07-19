use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub user_id: Uuid,
    pub device_id: Uuid,
    pub last_sync: DateTime<Utc>,
    pub sync_token: String,
    pub file_states: HashMap<String, FileState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileState {
    pub path: String,
    pub checksum: String,
    pub size: u64,
    pub modified_at: DateTime<Utc>,
    pub version: u64,
    pub sync_status: SyncStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStatus {
    Synced,
    Modified,
    Deleted,
    Conflicted,
    Uploading,
    Downloading,
    Error(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncRequest {
    pub device_id: Uuid,
    pub last_sync_token: Option<String>,
    pub changes: Vec<FileChange>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResponse {
    pub sync_token: String,
    pub changes: Vec<FileChange>,
    pub conflicts: Vec<SyncConflict>,
    pub deleted_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub change_type: ChangeType,
    pub metadata: Option<super::FileMetadata>,
    pub checksum: Option<String>,
    pub delta: Option<FileDelta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
    Moved { old_path: String },
    Copied { source_path: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDelta {
    pub blocks: Vec<DeltaBlock>,
    pub total_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaBlock {
    pub offset: u64,
    pub size: u64,
    pub checksum: String,
    pub data: Option<Vec<u8>>, // None for unchanged blocks
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflict {
    pub path: String,
    pub conflict_type: ConflictType,
    pub local_version: FileState,
    pub remote_version: FileState,
    pub base_version: Option<FileState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    ModifyModify,
    ModifyDelete,
    DeleteModify,
    MoveMove,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConflictResolution {
    pub path: String,
    pub resolution: ResolutionStrategy,
    pub resolved_metadata: Option<super::FileMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    UseLocal,
    UseRemote,
    UseBase,
    Merge,
    Rename { new_name: String },
    Custom { content: Vec<u8> },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncProgress {
    pub total_files: u64,
    pub completed_files: u64,
    pub total_bytes: u64,
    pub transferred_bytes: u64,
    pub current_file: Option<String>,
    pub speed_bps: u64,
    pub eta_seconds: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub device_type: DeviceType,
    pub platform: String,
    pub version: String,
    pub last_seen: DateTime<Utc>,
    pub is_active: bool,
    pub sync_folders: Vec<SyncFolder>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DeviceType {
    Desktop,
    Mobile,
    Web,
    Cli,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncFolder {
    pub local_path: String,
    pub remote_path: String,
    pub sync_direction: SyncDirection,
    pub is_active: bool,
    pub filters: SyncFilters,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SyncDirection {
    Bidirectional,
    UploadOnly,
    DownloadOnly,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncFilters {
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub max_file_size: Option<u64>,
    pub ignore_hidden: bool,
}
