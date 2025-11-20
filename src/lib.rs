pub mod communication;
pub mod configuration;
pub mod core;
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
