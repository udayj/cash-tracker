use crate::configuration::Context;
use crate::request::tools::ToolExecutor;
use crate::{database::DatabaseService, request::llm::LLMOrchestrator};
use std::sync::Arc;
use thiserror::Error;
mod llm;
mod tools;
pub mod types;
mod visualization;

use types::*;

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("Initialization Error")]
    InitializationError,

    #[error("LLM Error: {0}")]
    LLMError(#[from] crate::request::llm::LLMError),

    #[error("Tool Execution Error: {0}")]
    ToolError(#[from] crate::request::tools::ToolError),

    #[error("Database error: {0}")]
    DatabaseError(String),
}

pub struct RequestFulfilment {
    pub llm_service: LLMOrchestrator,
    pub database: Arc<DatabaseService>,
}

impl RequestFulfilment {
    pub async fn new(context: &Context) -> Result<Self, RequestError> {
        let llm_service = LLMOrchestrator::new(context)
            .await
            .map_err(|_| RequestError::InitializationError)?;
        let database = context.database.clone();
        Ok(RequestFulfilment {
            llm_service,
            database,
        })
    }

    pub async fn fulfil_request(
        &self,
        request: &str,
        ctx: &SessionContext,
    ) -> Result<FulfilmentResult, RequestError> {
        let categories = self
            .database
            .get_categories(ctx.user_id)
            .await
            .map_err(|e| RequestError::DatabaseError(e.to_string()))?;

        let full_request = {
            let mut parts = Vec::new();

            // Add category context if categories exist
            if !categories.is_empty() {
                parts.push(format!(
                    "AVAILABLE CATEGORIES: {}\nPrefer using existing categories when appropriate. Only create new categories if the expense doesn't fit any existing one.",
                    categories.join(", ")
                ));
            }

            // Add replied record context if exists
            if let Some(ref record_ctx) = ctx.replied_record {
                parts.push(Self::format_record_context(record_ctx));
            }

            // Add user request
            parts.push(format!("User request: {}", request));
            parts.join("\n\n")
        };

        // Get LLM response with tool calls
        let llm_response = self.llm_service.try_parse(&full_request).await?;

        if llm_response.tool_calls.is_empty() {
            return Ok(FulfilmentResult {
                response: "No action taken".to_string(),
                finalize: None,
                image: None,
            });
        }

        // Execute tool and generate response based on tool type
        let tool_executor = ToolExecutor::new(self.database.clone());
        let tool_call = &llm_response.tool_calls[0]; // Use first tool call

        let (record_id, response, image) = tool_executor
            .execute_tool(&tool_call.function.name, &tool_call.function.arguments, ctx)
            .await?;
        // Generate response message based on tool
        let finalize = match tool_call.function.name.as_str() {
            "add_cash" => record_id.map(|id| FinalizeAction {
                record_id: id,
                action_type: ActionType::CashTransaction,
            }),
            "add_expense" => record_id.map(|id| FinalizeAction {
                record_id: id,
                action_type: ActionType::Expense,
            }),
            _ => None,
        };

        Ok(FulfilmentResult {
            response,
            finalize,
            image,
        })
    }

    fn format_record_context(record: &RecordContext) -> String {
        match record {
            RecordContext::Expense(expense) => {
                format!(
                    "CONTEXT: The user is replying about an existing expense:\n\
                     - Expense ID: {}\n\
                     - Amount: ₹{}\n\
                     - Description: {}\n\
                     - Category: {}\n\
                     - Date: {}",
                    expense.id,
                    expense.amount,
                    expense.description,
                    expense.category,
                    expense.expense_date
                )
            }
            RecordContext::CashTransaction(cash) => {
                format!(
                    "CONTEXT: The user is replying about an existing cash transaction:\n\
                     - Transaction ID: {}\n\
                     - Amount: ₹{}\n\
                     - Date: {}",
                    cash.id, cash.amount, cash.transaction_date
                )
            }
        }
    }

    pub async fn finalize(
        &self,
        action: FinalizeAction,
        bot_message_id: i64,
    ) -> Result<(), RequestError> {
        match action.action_type {
            ActionType::Expense => {
                self.database
                    .update_expense_bot_message(action.record_id, bot_message_id)
                    .await
                    .map_err(|e| RequestError::DatabaseError(e.to_string()))?;
            }
            ActionType::CashTransaction => {
                self.database
                    .update_cash_bot_message(action.record_id, bot_message_id)
                    .await
                    .map_err(|e| RequestError::DatabaseError(e.to_string()))?;
            }
        }
        Ok(())
    }
}
