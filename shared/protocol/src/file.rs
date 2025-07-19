use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub parent_id: Option<Uuid>,
    pub owner_id: Uuid,
    pub size: u64,
    pub mime_type: String,
    pub checksum: String, // BLAKE3 hash
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub accessed_at: DateTime<Utc>,
    pub version: u64,
    pub is_directory: bool,
    pub is_encrypted: bool,
    pub permissions: FilePermissions,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>, // Extended attributes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePermissions {
    pub owner_permissions: Vec<FilePermission>,
    pub group_permissions: HashMap<Uuid, Vec<FilePermission>>,
    pub public_permissions: Vec<FilePermission>,
    pub inherit_permissions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilePermission {
    Read,
    Write,
    Delete,
    Share,
    Execute,
    ChangePermissions,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileUploadRequest {
    pub name: String,
    pub parent_path: String,
    pub size: u64,
    pub mime_type: String,
    pub checksum: Option<String>,
    pub chunk_size: Option<u64>,
    pub overwrite: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileUploadResponse {
    pub upload_id: Uuid,
    pub file_id: Option<Uuid>,
    pub chunk_urls: Vec<String>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileChunk {
    pub upload_id: Uuid,
    pub chunk_index: u32,
    pub chunk_size: u64,
    pub checksum: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileVersion {
    pub id: Uuid,
    pub file_id: Uuid,
    pub version_number: u64,
    pub size: u64,
    pub checksum: String,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub comment: Option<String>,
    pub is_current: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileShare {
    pub id: Uuid,
    pub file_id: Uuid,
    pub shared_by: Uuid,
    pub share_token: String,
    pub permissions: Vec<FilePermission>,
    pub expires_at: Option<DateTime<Utc>>,
    pub password_hash: Option<String>,
    pub download_count: u64,
    pub max_downloads: Option<u64>,
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DirectoryListing {
    pub path: String,
    pub files: Vec<FileMetadata>,
    pub total_size: u64,
    pub permissions: FilePermissions,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileSearchQuery {
    pub query: String,
    pub path: Option<String>,
    pub file_types: Vec<String>,
    pub size_min: Option<u64>,
    pub size_max: Option<u64>,
    pub modified_after: Option<DateTime<Utc>>,
    pub modified_before: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileSearchResult {
    pub files: Vec<FileMetadata>,
    pub total_count: u64,
    pub search_time_ms: u64,
}
