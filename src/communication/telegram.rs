use crate::configuration::Context;
use crate::core::Error;
use crate::core::Service;
use crate::database::{DatabaseService, UserStatus};
use crate::request::RequestFulfilment;
use crate::request::types::{RecordContext, SessionContext};
use async_trait::async_trait;
use std::env;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::InputFile;
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
    admin_id: i64,
}

#[async_trait]
impl Service for TelegramService {
    type Context = Context;

    async fn new(context: Context, error_channel: mpsc::Sender<String>) -> Self {
        let bot_token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not found");
        let bot = Bot::new(bot_token);
        let database = context.database.clone();
        let admin_id = context.config.admin_id;
        let request_fulfilment = RequestFulfilment::new(&context)
            .await
            .map_err(|_| TelegramServiceError::InitializationError)
            .unwrap();
        Self {
            bot,
            request_fulfilment,
            database,
            error_channel,
            admin_id,
        }
    }

    async fn run(self) -> Result<(), Error> {
        let error_channel = Arc::new(self.error_channel);
        let request_fulfilment = Arc::new(self.request_fulfilment);
        let database = self.database;
        let admin_id = self.admin_id;
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
                    admin_id,
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
        admin_id: i64,
    ) -> ResponseResult<()> {
        let chat_id = msg.chat.id;
        let user_id = chat_id.0;

        // Admin commands — checked before any auth logic
        if user_id == admin_id
            && let Some(text) = msg.text()
        {
            let parts: Vec<&str> = text.splitn(2, ' ').collect();
            match parts[0] {
                "/approve" | "/suspend" => {
                    let reply = if parts.len() < 2 || parts[1].trim().is_empty() {
                        format!("Usage: {} <user_id>", parts[0])
                    } else {
                        match parts[1].trim().parse::<i64>() {
                            Ok(target_id) => {
                                let (status, label) = if parts[0] == "/approve" {
                                    (UserStatus::Approved, "approved")
                                } else {
                                    (UserStatus::Suspended, "suspended")
                                };
                                match database.set_user_status(target_id, status).await {
                                    Ok(true) => format!("✅ User {} {}", target_id, label),
                                    Ok(false) => format!("User {} not found", target_id),
                                    Err(e) => {
                                        let _ = error_channel
                                            .send(format!("Failed to update user status: {}", e))
                                            .await;
                                        "Failed to update user status".to_string()
                                    }
                                }
                            }
                            Err(_) => "Invalid user ID — must be a number".to_string(),
                        }
                    };
                    let _ = bot.send_message(chat_id, reply).await;
                    return Ok(());
                }
                _ => {}
            }
        }

        // Authorization check (admin bypasses entirely)
        if user_id != admin_id {
            match database.get_user_status(user_id).await {
                Ok(Some(UserStatus::Approved)) => {}
                Ok(Some(UserStatus::Pending)) | Ok(Some(UserStatus::Suspended)) => {
                    return Ok(());
                }
                Ok(None) => {
                    if let Err(e) = database.insert_pending_user(user_id).await {
                        let _ = error_channel
                            .send(format!("Failed to insert pending user: {}", e))
                            .await;
                    }
                    let preview = msg.text().unwrap_or("(non-text message)");
                    let _ = error_channel
                        .send(format!(
                            "New access request\nUser ID: {}\nMessage: {}",
                            user_id, preview
                        ))
                        .await;
                    return Ok(());
                }
                Err(e) => {
                    let _ = error_channel.send(format!("Auth check error: {}", e)).await;
                    return Ok(());
                }
            }
        }

        // Handle /help
        if let Some(text) = msg.text()
            && text == "/help"
        {
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

        // Replied record is part of the context to ensure user's can reply to either their own message or the confirmation message
        // given by the bot to modify the action - add expense / add cash
        let replied_record = if let Some(reply_to) = msg.reply_to_message() {
            let replied_msg_id = reply_to.id.0 as i64;
            match database
                .find_expense_by_message(user_id, replied_msg_id)
                .await
            {
                Ok(Some(expense)) => Some(RecordContext::Expense(expense)),
                Ok(None) => match database.find_cash_by_message(user_id, replied_msg_id).await {
                    Ok(Some(cash)) => Some(RecordContext::CashTransaction(cash)),
                    Ok(None) => None,
                    Err(e) => {
                        let _ = error_channel
                            .send(format!("Database lookup error: {}", e))
                            .await;
                        None
                    }
                },
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
                    let sent_msg_result = if let Some(image_data) = result.image {
                        let photo = InputFile::memory(image_data);
                        bot.send_photo(chat_id, photo)
                            .caption(result.response)
                            .await
                            .map(|msg| msg.id)
                    } else {
                        bot.send_message(chat_id, result.response)
                            .await
                            .map(|msg| msg.id)
                    };

                    match sent_msg_result {
                        Ok(sent_msg_id) => {
                            // The finalize action is used to set the bot reply message id for an action
                            // This enables us to later refer to it if the user replies to the bot message to modify an earlier action
                            if let Some(finalize_action) = result.finalize {
                                let bot_msg_id = sent_msg_id.0 as i64;
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
