use cash_tracker::AppError;
use cash_tracker::communication::{ErrorAlertService, TelegramService};
use cash_tracker::configuration::Context;
use cash_tracker::core::ServiceManager;
use dotenvy::dotenv;
use std::env;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use tracing::{Level, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenv().ok();
    let config_file = env::var("CONFIG_FILE").unwrap_or_else(|_| "config.json".to_string());
    let context = Context::new(&config_file)
        .await
        .map_err(|e| AppError::ConfigError(e.to_string()))?;

    let log_level = Level::from_str(&context.config.log_level).unwrap_or(Level::INFO);
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::new(log_level.to_string()))
        .init();
    info!("Starting Assistant Application");

    let (error_sender, error_receiver) = mpsc::channel::<String>(100);
    let shared_error_receiver = Arc::new(Mutex::new(error_receiver));
    let mut service_manager = ServiceManager::new(context);
    service_manager.spawn_with_error_receiver::<ErrorAlertService>(shared_error_receiver);
    service_manager.spawn::<TelegramService>(error_sender);
    service_manager
        .wait()
        .await
        .map_err(|_| AppError::ServiceError)
}
