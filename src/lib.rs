pub mod core;
pub mod configuration;
pub mod communication;
pub mod database;
pub mod request;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Config Error:{0}")]
    ConfigError(String),

    #[error("Service error")]
    ServiceError,
}
