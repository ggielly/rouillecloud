pub mod server;
pub mod database;
pub mod storage;
pub mod auth;
pub mod monitoring;

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: server::ServerConfig,
    pub database: database::DatabaseConfig,
    pub storage: storage::StorageConfig,
    pub auth: auth::AuthConfig,
    pub monitoring: monitoring::MonitoringConfig,
}

impl AppConfig {
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = tokio::fs::read_to_string(path).await
            .map_err(ConfigError::Io)?;
        
        let config: AppConfig = toml::from_str(&content)
            .map_err(ConfigError::Parse)?;
        
        config.validate()?;
        Ok(config)
    }
    
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.server.validate()?;
        self.database.validate()?;
        self.storage.validate()?;
        self.auth.validate()?;
        self.monitoring.validate()?;
        Ok(())
    }
    
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut config = Self::default();
        
        if let Ok(host) = std::env::var("SERVER_HOST") {
            config.server.host = host;
        }
        
        if let Ok(port) = std::env::var("SERVER_PORT") {
            config.server.port = port.parse()
                .map_err(|_| ConfigError::InvalidValue("SERVER_PORT must be a valid port number".to_string()))?;
        }
        
        if let Ok(db_url) = std::env::var("DATABASE_URL") {
            config.database.url = db_url;
        }
        
        if let Ok(storage_path) = std::env::var("STORAGE_PATH") {
            config.storage.local_path = Some(storage_path);
        }
        
        if let Ok(jwt_secret) = std::env::var("JWT_SECRET") {
            config.auth.jwt_secret = jwt_secret;
        }
        
        config.validate()?;
        Ok(config)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: server::ServerConfig::default(),
            database: database::DatabaseConfig::default(),
            storage: storage::StorageConfig::default(),
            auth: auth::AuthConfig::default(),
            monitoring: monitoring::MonitoringConfig::default(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),
    #[error("Missing required configuration: {0}")]
    MissingRequired(String),
}
