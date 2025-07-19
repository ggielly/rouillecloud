use serde::{Deserialize, Serialize};
use crate::config::ConfigError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
    pub max_connections: usize,
    pub keep_alive: u64,
    pub client_timeout: u64,
    pub client_shutdown: u64,
    pub tls: Option<TlsConfig>,
    pub cors: CorsConfig,
    pub rate_limiting: RateLimitConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub cert_file: String,
    pub key_file: String,
    pub ca_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub expose_headers: Vec<String>,
    pub max_age: Option<usize>,
    pub allow_credentials: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub cleanup_interval: u64,
}

impl ServerConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.host.is_empty() {
            return Err(ConfigError::InvalidValue("Host cannot be empty".to_string()));
        }
        
        if self.port == 0 {
            return Err(ConfigError::InvalidValue("Port must be greater than 0".to_string()));
        }
        
        if self.max_connections == 0 {
            return Err(ConfigError::InvalidValue("Max connections must be greater than 0".to_string()));
        }
        
        if let Some(ref tls) = self.tls {
            if tls.cert_file.is_empty() {
                return Err(ConfigError::InvalidValue("TLS cert file cannot be empty".to_string()));
            }
            if tls.key_file.is_empty() {
                return Err(ConfigError::InvalidValue("TLS key file cannot be empty".to_string()));
            }
        }
        
        Ok(())
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            workers: None, // Use system default
            max_connections: 25000,
            keep_alive: 75,
            client_timeout: 5000,
            client_shutdown: 5000,
            tls: None,
            cors: CorsConfig::default(),
            rate_limiting: RateLimitConfig::default(),
        }
    }
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec![
                "http://localhost:3000".to_string(),
                "http://127.0.0.1:3000".to_string(),
            ],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "OPTIONS".to_string(),
                "PROPFIND".to_string(),
                "PROPPATCH".to_string(),
                "MKCOL".to_string(),
                "COPY".to_string(),
                "MOVE".to_string(),
                "LOCK".to_string(),
                "UNLOCK".to_string(),
            ],
            allowed_headers: vec![
                "authorization".to_string(),
                "accept".to_string(),
                "content-type".to_string(),
                "x-requested-with".to_string(),
                "depth".to_string(),
                "destination".to_string(),
                "if".to_string(),
                "lock-token".to_string(),
                "overwrite".to_string(),
                "timeout".to_string(),
            ],
            expose_headers: vec![],
            max_age: Some(3600),
            allow_credentials: true,
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            requests_per_minute: 1000,
            burst_size: 100,
            cleanup_interval: 60,
        }
    }
}
