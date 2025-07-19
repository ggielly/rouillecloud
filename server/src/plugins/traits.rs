//! Core plugin traits for extensibility

use async_trait::async_trait;
use std::any::Any;
use std::fmt::Debug;

/// Base trait for all plugins
pub trait Plugin: Debug + Send + Sync {
    fn name(&self) -> &str;
    fn as_any(&self) -> &dyn Any;
}

/// Authentication plugin trait
#[async_trait]
pub trait AuthPlugin: Plugin {
    async fn authenticate(&self, username: &str, password: &str) -> Result<bool, String>;
}

/// Monitoring plugin trait
#[async_trait]
pub trait MonitoringPlugin: Plugin {
    async fn record_metric(&self, metric: &str, value: f64);
}

/// Storage backend plugin trait
#[async_trait]
pub trait StoragePlugin: Plugin {
    async fn store_file(&self, path: &str, data: &[u8]) -> Result<(), String>;
    async fn retrieve_file(&self, path: &str) -> Result<Vec<u8>, String>;
}
