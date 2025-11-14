use crate::configuration::Context;
use crate::core::Error;
use crate::core::Service;
use crate::request::RequestFulfilment;
use async_trait::async_trait;
use std::env;
use std::sync::Arc;
use teloxide::prelude::*;
use thiserror::Error;
use tokio::sync::mpsc;

#[derive(Debug, Error)]
pub enum TelegramServiceError {
    #[error("Initialization Error")]
    InitializationError,
}

pub struct TelegramService {
    bot: Bot,
    request_fulfilment: RequestFulfilment,
    error_channel: mpsc::Sender<String>,
}

#[async_trait]
impl Service for TelegramService {
    type Context = Context;

    async fn new(context: Context, error_channel: mpsc::Sender<String>) -> Self {
        let bot_token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not found");
        let bot = Bot::new(bot_token);
        let request_fulfilment = RequestFulfilment::new(&context)
            .await
            .map_err(|_| TelegramServiceError::InitializationError).unwrap();
        Self {
            bot,
            request_fulfilment,
            error_channel,
        }
    }

    async fn run(self) -> Result<(), Error> {
        let error_channel = Arc::new(self.error_channel);
        let request_fulfilment = Arc::new(self.request_fulfilment);
        teloxide::repl(self.bot, move |bot: Bot, msg: Message| {
            let error_channel = Arc::clone(&error_channel);
            let request_fulfilment = Arc::clone(&request_fulfilment);
            async move {
                tokio::spawn(Self::handle_message(bot, msg, request_fulfilment, error_channel));
                respond(())
            }
        })
        .await;
        Ok(())
    }
}

impl TelegramService {
    async fn handle_message(
        bot: Bot,
        msg: Message,
        request_fulfilment: Arc<RequestFulfilment>,
        error_channel: Arc<mpsc::Sender<String>>,
    ) -> ResponseResult<()> {

        let chat_id = msg.chat.id;
        if let Some(request) = msg.text() {
            let response = request_fulfilment.fulfil_request(request).await.unwrap();
            let _ = bot.send_message(chat_id, response);
        }
        /*let chat_id = msg.chat.id;
        let telegram_id = chat_id.0.to_string();
        println!("chat id:{}", chat_id);
        println!("telegram_id:{}", telegram_id);
        println!("message id:{}", msg.id);
        if let Some(message) = msg.reply_to_message() {
            println!("earlier message id:{}", message.id);
            println!("chat id:{}", message.chat.id);
            println!("telegram_id:{}", message.chat.id.0.to_string());
        }*/
        return Ok(());
    }
}
