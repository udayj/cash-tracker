use crate::{database::DatabaseService, request::llm::LLMOrchestrator};
use std::sync::Arc;
use thiserror::Error;
use crate::configuration::Context;
use crate::request::tools::{ToolExecutor, SessionContext};

pub mod llm;
pub mod tools;

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("Initialization Error")]
    InitializationError,

    #[error("LLM Error: {0}")]
    LLMError(#[from] crate::request::llm::LLMError),

    #[error("Tool Execution Error: {0}")]
    ToolError(#[from] crate::request::tools::ToolError),

    #[error("Database Error: {0}")]
    DatabaseError(String),
}

#[derive(Debug, Clone)]
pub enum ActionType {
    Expense,
    CashTransaction,
}

#[derive(Debug, Clone)]
pub struct FinalizeAction {
    pub record_id: i64,
    pub action_type: ActionType,
}

pub struct FulfilmentResult {
    pub response: String,
    pub finalize: Option<FinalizeAction>,
}

pub struct RequestFulfilment {

    pub llm_service: LLMOrchestrator,
    pub database: Arc<DatabaseService>
}

impl RequestFulfilment {

    pub async fn new(context: &Context) -> Result<Self, RequestError> {

        let llm_service = LLMOrchestrator::new(context).await.map_err(|_| RequestError::InitializationError)?;
        let database = context.database.clone();
        Ok(RequestFulfilment {
            llm_service,
            database
        })
    }

    pub async fn fulfil_request(&self, request: &str, ctx: &SessionContext) -> Result<FulfilmentResult, RequestError> {
        // Get LLM response with tool calls
        let llm_response = self.llm_service.try_parse(request).await?;

        if llm_response.tool_calls.is_empty() {
            return Ok(FulfilmentResult {
                response: "No action taken".to_string(),
                finalize: None,
            });
        }

        // Execute tool and generate response based on tool type
        let tool_executor = ToolExecutor::new(self.database.clone());
        let tool_call = &llm_response.tool_calls[0]; // Use first tool call

        let record_id = tool_executor
            .execute_tool(&tool_call.function.name, &tool_call.function.arguments, ctx)
            .await?;

        // Generate response message based on tool
        let response = self.format_response(&tool_call.function.name, &tool_call.function.arguments)?;

        // Determine if finalization is needed
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

        Ok(FulfilmentResult { response, finalize })
    }

    pub async fn finalize(&self, action: FinalizeAction, bot_message_id: i64) -> Result<(), RequestError> {
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

    fn format_response(&self, tool_name: &str, arguments: &str) -> Result<String, RequestError> {
        use crate::request::tools::*;

        match tool_name {
            "add_cash" => {
                let args: AddCashArgs = serde_json::from_str(arguments)
                    .map_err(|e| ToolError::ArgumentParseError(e.to_string()))?;
                Ok(format!("✅ Added ₹{} to cash balance", args.amount))
            }
            "add_expense" => {
                let args: AddExpenseArgs = serde_json::from_str(arguments)
                    .map_err(|e| ToolError::ArgumentParseError(e.to_string()))?;
                Ok(format!("✅ Added ₹{} for {} under {}", args.amount, args.description, args.category))
            }
            "modify_expense" => {
                Ok("✅ Expense modified successfully".to_string())
            }
            "delete_expense" => {
                Ok("✅ Expense deleted successfully".to_string())
            }
            "get_balance" => {
                Ok("Balance retrieved".to_string()) // Will be replaced with actual balance
            }
            "get_expense_breakdown" => {
                Ok("Expense breakdown retrieved".to_string()) // Will be replaced with actual data
            }
            "get_category_expenses" => {
                Ok("Category expenses retrieved".to_string()) // Will be replaced with actual data
            }
            "get_categories" => {
                Ok("Categories retrieved".to_string()) // Will be replaced with actual data
            }
            _ => Ok("Action completed".to_string()),
        }
    }
}