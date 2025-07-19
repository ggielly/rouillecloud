use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub local_path: Option<String>,
    pub max_file_size: u64,
    pub allowed_extensions: Vec<String>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            local_path: Some("./storage".to_string()),
            max_file_size: 100 * 1024 * 1024, // 100MB
            allowed_extensions: vec![
                "txt".to_string(),
                "pdf".to_string(),
                "jpg".to_string(),
                "png".to_string(),
            ],
        }
    }
}

impl StorageConfig {
    pub fn validate(&self) -> Result<(), super::ConfigError> {
        if self.max_file_size == 0 {
            return Err(super::ConfigError::InvalidValue(
                "max_file_size must be greater than 0".to_string()
            ));
        }
        Ok(())
    }

    pub fn from_env() -> Result<Self, super::ConfigError> {
        let mut config = Self::default();

        if let Ok(path) = std::env::var("STORAGE_LOCAL_PATH") {
            config.local_path = Some(path);
        }

        if let Ok(size) = std::env::var("STORAGE_MAX_FILE_SIZE") {
            config.max_file_size = size.parse().map_err(|_| {
                super::ConfigError::InvalidValue("STORAGE_MAX_FILE_SIZE must be a valid number".to_string())
            })?;
        }

        if let Ok(extensions) = std::env::var("STORAGE_ALLOWED_EXTENSIONS") {
            config.allowed_extensions = extensions.split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }

            config.validate()?;
            Ok(config)
        }
    }
