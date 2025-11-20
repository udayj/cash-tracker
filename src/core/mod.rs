use thiserror::Error;
mod cache;
mod http;
mod service;
mod service_manager;

pub use cache::ExpirableCache;
pub use http::RetryableClient;
pub use service::{Service, ServiceWithReceiver};
pub use service_manager::ServiceManager;

#[derive(Error, Debug)]
#[error("{0}")]
pub struct Error(String);

impl Error {
    pub fn new(s: &str) -> Error {
        Error(s.to_string())
    }

    pub fn from<E: std::error::Error>(e: E) -> Self {
        Self(e.to_string())
    }
}
