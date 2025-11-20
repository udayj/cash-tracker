use super::Error;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, Sender};

#[async_trait]
pub trait Service {
    type Context: Clone + Send;
    async fn new(context: Self::Context, error_channel: Sender<String>) -> Self;
    async fn run(self) -> Result<(), Error>;
}

#[async_trait]
pub trait ServiceWithReceiver {
    type Context: Clone + Send;
    async fn new(context: Self::Context, receiver: Option<Arc<Mutex<Receiver<String>>>>) -> Self;
    async fn run(self) -> Result<(), Error>;
}
