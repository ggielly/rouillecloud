use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

// WebDAV Protocol Implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebDavResource {
    pub href: String,
    pub properties: HashMap<String, WebDavProperty>,
    pub status: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebDavProperty {
    DisplayName(String),
    CreationDate(DateTime<Utc>),
    LastModified(DateTime<Utc>),
    ContentLength(u64),
    ContentType(String),
    ETag(String),
    ResourceType(ResourceType),
    LockDiscovery(Vec<ActiveLock>),
    SupportedLock(Vec<LockType>),
    Custom { namespace: String, name: String, value: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    Collection,
    File,
    Principal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveLock {
    pub lock_type: LockType,
    pub lock_scope: LockScope,
    pub depth: Depth,
    pub owner: Option<String>,
    pub timeout: Option<DateTime<Utc>>,
    pub lock_token: String,
    pub lock_root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LockType {
    Write,
    Read,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LockScope {
    Exclusive,
    Shared,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Depth {
    Zero,
    One,
    Infinity,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PropFindRequest {
    pub properties: PropFindType,
    pub depth: Depth,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PropFindType {
    AllProp,
    PropName,
    Prop(Vec<String>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PropPatchRequest {
    pub updates: Vec<PropertyUpdate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PropertyUpdate {
    pub action: PropertyAction,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PropertyAction {
    Set,
    Remove,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LockRequest {
    pub lock_info: LockInfo,
    pub depth: Depth,
    pub timeout: Option<u32>, // seconds
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LockInfo {
    pub lock_scope: LockScope,
    pub lock_type: LockType,
    pub owner: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CopyMoveRequest {
    pub destination: String,
    pub overwrite: bool,
    pub depth: Depth,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MultiStatus {
    pub responses: Vec<WebDavResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebDavResponse {
    pub href: String,
    pub status: Option<u16>,
    pub prop_stats: Vec<PropStat>,
    pub error: Option<String>,
    pub response_description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PropStat {
    pub properties: HashMap<String, WebDavProperty>,
    pub status: u16,
    pub error: Option<String>,
}
