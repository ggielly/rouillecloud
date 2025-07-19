use crate::config::DatabaseConfig;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

#[derive(Clone)]
pub struct DatabasePool {
    pool: Pool<Postgres>,
}

impl DatabasePool {
    pub async fn new(config: &DatabaseConfig) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections.unwrap_or(10))
            .connect(&config.url)
            .await?;

        Ok(DatabasePool { pool })
    }

    pub async fn migrate(&self) -> Result<(), sqlx::Error> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| sqlx::Error::Migrate(Box::new(e)))
    }

    pub fn pool(&self) -> &Pool<Postgres> {
        &self.pool
    }
}

use serde::{Deserialize, Serialize};

use super::ConfigError;

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct DatabaseConfig {
    pub url: String,

    pub max_connections: u32,

    pub timeout_seconds: u64,
}

impl DatabaseConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.url.is_empty() {
            return Err(ConfigError::MissingRequired("database.url".to_string()));
        }

        Ok(())
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite://./data.db".to_string(),

            max_connections: 10,

            timeout_seconds: 30,
        }
    }
}
