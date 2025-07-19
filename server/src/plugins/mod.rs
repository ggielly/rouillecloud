//! Plugin system module

pub mod manager;
pub mod traits;
pub mod auth_local;
pub mod monitoring_prometheus;
pub mod storage_local;
pub mod dynamic_loader;

// Re-export traits for convenience
pub use traits::*;
