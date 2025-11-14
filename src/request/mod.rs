use crate::{database::DatabaseService, request::llm::LLMOrchestrator};
use std::sync::Arc;
use thiserror::Error;
use crate::configuration::Context;
use crate::request::tools::ToolExecutor;

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

    pub async fn fulfil_request(&self, request: &str) -> Result<String, RequestError> {
        // Get LLM response with tool calls
        let llm_response = self.llm_service.try_parse(request).await?;

        if llm_response.tool_calls.is_empty() {
            return Ok("No action taken".to_string());
        }

        // Execute tool and generate response based on tool type
        let tool_executor = ToolExecutor::new(self.database.clone());
        let tool_call = &llm_response.tool_calls[0]; // Use first tool call

        tool_executor
            .execute_tool(&tool_call.function.name, &tool_call.function.arguments)
            .await?;

        // Generate response message based on tool
        let response = self.format_response(&tool_call.function.name, &tool_call.function.arguments)?;
        Ok(response)
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