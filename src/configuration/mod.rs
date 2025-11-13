use serde::Deserialize;
use std::fs;
use std::sync::Arc;
use thiserror::Error;

use crate::database::DatabaseService;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("File read error")]
    FileError,

    #[error("Deserialization error:{0}")]
    DeserializationError(String),

    #[error("Database init error")]
    DatabaseServiceInitError,
}

#[derive(Debug, Deserialize, Clone)]

pub struct Config {
    pub log_level: String,
    pub db_url: String,
    pub telegram: TelegramConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TelegramConfig {
    pub price_alert_subscribers: Vec<i64>,
    pub error_channel_id: i64,
    pub admin_telegram_id: String,
}

#[derive(Clone)]
pub struct Context {
    pub config: Config,
    pub database: Arc<DatabaseService>,
}

impl Context {
    pub async fn new(config_file: &str) -> Result<Self, ConfigError> {
        let config = Config::new(config_file)?;
        let database = Arc::new(
            DatabaseService::new(config.db_url.clone())
                .await
                .map_err(|_| ConfigError::DatabaseServiceInitError)?,
        );
        Ok(Self { config, database })
    }
}

impl Config {
    pub fn new(config_file: &str) -> Result<Self, ConfigError> {
        let config_str = fs::read_to_string(config_file).map_err(|_| ConfigError::FileError)?;
        let config: Config = serde_json::from_str(&config_str)
            .map_err(|e| ConfigError::DeserializationError(e.to_string()))?;
        Ok(config)
    }
}
