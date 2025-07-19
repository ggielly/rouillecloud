//! Plugin manager for dynamic loading and hot-reloading

use super::traits::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Default)]
pub struct PluginManager {
    auth_plugins: RwLock<HashMap<String, Arc<dyn AuthPlugin>>>,
    monitoring_plugins: RwLock<HashMap<String, Arc<dyn MonitoringPlugin>>>,
    storage_plugins: RwLock<HashMap<String, Arc<dyn StoragePlugin>>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_auth_plugin(&self, plugin: Arc<dyn AuthPlugin>) {
        self.auth_plugins.write().unwrap().insert(plugin.name().to_string(), plugin);
    }
    pub fn register_monitoring_plugin(&self, plugin: Arc<dyn MonitoringPlugin>) {
        self.monitoring_plugins.write().unwrap().insert(plugin.name().to_string(), plugin);
    }
    pub fn register_storage_plugin(&self, plugin: Arc<dyn StoragePlugin>) {
        self.storage_plugins.write().unwrap().insert(plugin.name().to_string(), plugin);
    }
    // TODO: Add dynamic loading/hot-reloading logic
}
