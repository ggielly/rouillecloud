use serde::{Deserialize, Serialize};
use crate::config::ConfigError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub enabled: bool,
    pub metrics_port: u16,
    pub log_level: String,
}

impl MonitoringConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.metrics_port == 0 {
            return Err(ConfigError::InvalidValue("metrics_port must be greater than 0".to_string()));
        }
        Ok(())
    }

    pub fn from_env() -> Result<Self, ConfigError> {
        let mut config = Self::default();

        if let Ok(enabled) = std::env::var("MONITORING_ENABLED") {
            config.enabled = enabled.parse().map_err(|_| {
                ConfigError::InvalidValue("MONITORING_ENABLED must be a boolean".to_string())
            })?;
        }

        if let Ok(port) = std::env::var("MONITORING_PORT") {
            config.metrics_port = port.parse().map_err(|_| {

    pub fn from_env() -> Result<Self, ConfigError> {
        let mut config = Self::default();

        if let Ok(enabled) = std::env::var("MONITORING_ENABLED") {
            config.enabled = enabled.parse().map_err(|_| {
                ConfigError::InvalidValue("MONITORING_ENABLED must be a boolean".to_string())
            })?;
        }

        if let Ok(port) = std::env::var("MONITORING_PORT") {
            config.metrics_port = port.parse().map_err(|_| {
                ConfigError::InvalidValue("MONITORING_PORT must be a valid port number".to_string())
            })?;
        }

        if let Ok(log_level) = std::env::var("MONITORING_LOG_LEVEL") {
            config.log_level = log_level;
        }

        config.validate()?;
        Ok(config)
    }
        ConfigError::InvalidValue("MONITORING_PORT must be a valid port number".to_string())
            })?;
        }

        if let Ok(log_level) = std::env::var("MONITORING_LOG_LEVEL") {
            config.log_level = log_level;
        }

        config.validate()?;
        Ok(config)
    }