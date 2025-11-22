use crate::configuration::Context;
use crate::core::Error;
use crate::core::Service;
use crate::database::DatabaseService;
use crate::request::RequestFulfilment;
use crate::request::types::{RecordContext, SessionContext};
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
    database: Arc<DatabaseService>,
    error_channel: mpsc::Sender<String>,
}

#[async_trait]
impl Service for TelegramService {
    type Context = Context;

    async fn new(context: Context, error_channel: mpsc::Sender<String>) -> Self {
        let bot_token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not found");
        let bot = Bot::new(bot_token);
        let database = context.database.clone();
        let request_fulfilment = RequestFulfilment::new(&context)
            .await
            .map_err(|_| TelegramServiceError::InitializationError)
            .unwrap();
        Self {
            bot,
            request_fulfilment,
            database,
            error_channel,
        }
    }

    async fn run(self) -> Result<(), Error> {
        let error_channel = Arc::new(self.error_channel);
        let request_fulfilment = Arc::new(self.request_fulfilment);
        let database = self.database;
        teloxide::repl(self.bot, move |bot: Bot, msg: Message| {
            let error_channel = Arc::clone(&error_channel);
            let request_fulfilment = Arc::clone(&request_fulfilment);
            let database = database.clone();
            async move {
                tokio::spawn(Self::handle_message(
                    bot,
                    msg,
                    request_fulfilment,
                    database,
                    error_channel,
                ));
                respond(())
            }
        })
        .await;
        Ok(())
    }
}

impl TelegramService {
    fn get_help_text() -> Result<String, std::io::Error> {
        std::fs::read_to_string("assets/help.txt")
    }

    async fn handle_message(
        bot: Bot,
        msg: Message,
        request_fulfilment: Arc<RequestFulfilment>,
        database: Arc<DatabaseService>,
        error_channel: Arc<mpsc::Sender<String>>,
    ) -> ResponseResult<()> {
        let chat_id = msg.chat.id;
        let user_id = chat_id.0;

        // Handle /help command
        if let Some(text) = msg.text() {
            if text == "/help" {
                match Self::get_help_text() {
                    Ok(help_text) => {
                        let _ = bot.send_message(chat_id, help_text).await;
                        return Ok(());
                    }
                    Err(e) => {
                        let _ = error_channel
                            .send(format!("Failed to read help file: {}", e))
                            .await;
                    }
                }
            }
        }
        let replied_record = if let Some(reply_to) = msg.reply_to_message() {
            let replied_msg_id = reply_to.id.0 as i64;

            // Try to find expense first
            match database
                .find_expense_by_message(user_id, replied_msg_id)
                .await
            {
                Ok(Some(expense)) => Some(RecordContext::Expense(expense)),
                Ok(None) => {
                    // Try to find cash transaction
                    match database.find_cash_by_message(user_id, replied_msg_id).await {
                        Ok(Some(cash)) => Some(RecordContext::CashTransaction(cash)),
                        Ok(None) => None,
                        Err(e) => {
                            let _ = error_channel
                                .send(format!("Database lookup error: {}", e))
                                .await;
                            None
                        }
                    }
                }
                Err(e) => {
                    let _ = error_channel
                        .send(format!("Database lookup error: {}", e))
                        .await;
                    None
                }
            }
        } else {
            None
        };
        let session_context = SessionContext {
            user_id: chat_id.0,
            user_message_id: msg.id.0 as i64,
            replied_record,
        };
        if let Some(request) = msg.text() {
            match request_fulfilment
                .fulfil_request(request, &session_context)
                .await
            {
                Ok(result) => {
                    // Send the response message
                    match bot.send_message(chat_id, result.response).await {
                        Ok(sent_msg) => {
                            // If there's a finalize action, execute it with the bot message ID
                            if let Some(finalize_action) = result.finalize {
                                let bot_msg_id = sent_msg.id.0 as i64;
                                if let Err(e) = request_fulfilment
                                    .finalize(finalize_action, bot_msg_id)
                                    .await
                                {
                                    let _ = error_channel
                                        .send(format!("Finalization error: {}", e))
                                        .await;
                                }
                            }
                        }
                        Err(e) => {
                            let _ = error_channel
                                .send(format!("Failed to send message: {}", e))
                                .await;
                        }
                    }
                }
                Err(e) => {
                    let _ = error_channel
                        .send(format!("Request fulfilment error: {}", e))
                        .await;
                    let _ = bot
                        .send_message(
                            chat_id,
                            "Sorry, something went wrong processing your request.",
                        )
                        .await;
                }
            }
        }
        Ok(())
    }
}
