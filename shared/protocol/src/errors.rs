use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiError {
    // Authentication errors
    Unauthorized,
    Forbidden,
    InvalidCredentials,
    TokenExpired,
    MfaRequired,
    
    // File operation errors
    FileNotFound,
    FileAlreadyExists,
    InsufficientStorage,
    FileTooLarge,
    InvalidFileName,
    AccessDenied,
    
    // Sync errors
    SyncConflict,
    InvalidSyncToken,
    DeviceNotFound,
    
    // WebDAV errors
    WebDavMethodNotAllowed,
    WebDavPreconditionFailed,
    WebDavLocked,
    WebDavConflict,
    
    // CalDAV errors
    CalendarNotFound,
    EventNotFound,
    InvalidCalendarData,
    CalendarConflict,
    
    // System errors
    InternalError,
    ServiceUnavailable,
    DatabaseError,
    StorageError,
    NetworkError,
    
    // Validation errors
    ValidationError { field: String, message: String },
    InvalidRequest { message: String },
    
    // Rate limiting
    RateLimitExceeded,
    
    // Custom error
    Custom { code: String, message: String },
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Unauthorized => write!(f, "Unauthorized access"),
            ApiError::Forbidden => write!(f, "Access forbidden"),
            ApiError::InvalidCredentials => write!(f, "Invalid username or password"),
            ApiError::TokenExpired => write!(f, "Authentication token has expired"),
            ApiError::MfaRequired => write!(f, "Multi-factor authentication required"),
            
            ApiError::FileNotFound => write!(f, "File not found"),
            ApiError::FileAlreadyExists => write!(f, "File already exists"),
            ApiError::InsufficientStorage => write!(f, "Insufficient storage space"),
            ApiError::FileTooLarge => write!(f, "File is too large"),
            ApiError::InvalidFileName => write!(f, "Invalid file name"),
            ApiError::AccessDenied => write!(f, "Access denied"),
            
            ApiError::SyncConflict => write!(f, "Synchronization conflict detected"),
            ApiError::InvalidSyncToken => write!(f, "Invalid synchronization token"),
            ApiError::DeviceNotFound => write!(f, "Device not found"),
            
            ApiError::WebDavMethodNotAllowed => write!(f, "WebDAV method not allowed"),
            ApiError::WebDavPreconditionFailed => write!(f, "WebDAV precondition failed"),
            ApiError::WebDavLocked => write!(f, "Resource is locked"),
            ApiError::WebDavConflict => write!(f, "WebDAV conflict"),
            
            ApiError::CalendarNotFound => write!(f, "Calendar not found"),
            ApiError::EventNotFound => write!(f, "Event not found"),
            ApiError::InvalidCalendarData => write!(f, "Invalid calendar data"),
            ApiError::CalendarConflict => write!(f, "Calendar conflict"),
            
            ApiError::InternalError => write!(f, "Internal server error"),
            ApiError::ServiceUnavailable => write!(f, "Service temporarily unavailable"),
            ApiError::DatabaseError => write!(f, "Database error"),
            ApiError::StorageError => write!(f, "Storage error"),
            ApiError::NetworkError => write!(f, "Network error"),
            
            ApiError::ValidationError { field, message } => {
                write!(f, "Validation error in field '{}': {}", field, message)
            }
            ApiError::InvalidRequest { message } => write!(f, "Invalid request: {}", message),
            
            ApiError::RateLimitExceeded => write!(f, "Rate limit exceeded"),
            
            ApiError::Custom { code, message } => write!(f, "{}: {}", code, message),
        }
    }
}

impl std::error::Error for ApiError {}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ApiError,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub request_id: Option<String>,
}

impl ErrorResponse {
    pub fn new(error: ApiError) -> Self {
        Self {
            message: error.to_string(),
            error,
            details: None,
            timestamp: chrono::Utc::now(),
            request_id: None,
        }
    }
    
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
    
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
