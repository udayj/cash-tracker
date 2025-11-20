use crate::configuration::Context;
use crate::core::{Error, ServiceWithReceiver};
use async_trait::async_trait;
use dotenvy::dotenv;
use std::env;
use std::sync::Arc;
use teloxide::prelude::*;
use tokio::sync::{Mutex, mpsc};
use tracing::error;

pub struct ErrorAlertService {
    bot: Bot,
    receiver: Option<Arc<Mutex<mpsc::Receiver<String>>>>,
    channel_id: i64,
}

#[async_trait]
impl ServiceWithReceiver for ErrorAlertService {
    type Context = Context;

    async fn new(_context: Context, receiver: Option<Arc<Mutex<mpsc::Receiver<String>>>>) -> Self {
        dotenv().ok();
        let error_bot_token = env::var("ERROR_BOT_TOKEN").expect("ERROR_BOT_TOKEN not found");
        let bot = Bot::new(error_bot_token);
        let channel_id =
            env::var("TELEGRAM_ERROR_CHANNEL_ID").expect("TELEGRAM_ERROR_CHANNEL_ID not found");

        Self {
            bot,
            receiver,
            channel_id: channel_id
                .parse::<i64>()
                .expect("Cannot parse error channel id"),
        }
    }

    async fn run(self) -> Result<(), Error> {
        if let Some(receiver) = &self.receiver {
            loop {
                let mut rx = receiver.lock().await;
                if let Some(error_message) = rx.recv().await {
                    drop(rx);
                    if let Err(e) = self
                        .bot
                        .send_message(ChatId(self.channel_id), &error_message)
                        .await
                    {
                        error!(error = %e, "Failed to send error alert");
                    }
                }
            }
        }
        Ok(())
    }
}
