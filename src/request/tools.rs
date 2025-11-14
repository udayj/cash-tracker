use crate::database::DatabaseService;
use serde::Deserialize;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ToolError {
    #[error("Unknown tool: {0}")]
    UnknownTool(String),

    #[error("Failed to parse tool arguments: {0}")]
    ArgumentParseError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),
}

// Tool argument structs
#[derive(Debug, Deserialize)]
pub struct AddCashArgs {
    pub amount: f64,
    pub date: String,
}

#[derive(Debug, Deserialize)]
pub struct AddExpenseArgs {
    pub amount: f64,
    pub description: String,
    pub category: String,
    pub date: String,
}

#[derive(Debug, Deserialize)]
pub struct ModifyExpenseArgs {
    pub expense_id: i64,
    pub field: String,
    pub new_value: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteExpenseArgs {
    pub expense_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct GetExpenseBreakdownArgs {
    pub start_date: String,
    pub end_date: String,
}

#[derive(Debug, Deserialize)]
pub struct GetCategoryExpensesArgs {
    pub category: String,
    pub start_date: String,
    pub end_date: String,
}

pub struct ToolExecutor {
    database: Arc<DatabaseService>,
}

impl ToolExecutor {
    pub fn new(database: Arc<DatabaseService>) -> Self {
        Self { database }
    }

    pub async fn execute_tool(
        &self,
        tool_name: &str,
        arguments: &str,
    ) -> Result<String, ToolError> {
        match tool_name {
            "add_cash" => {
                let args: AddCashArgs = serde_json::from_str(arguments)
                    .map_err(|e| ToolError::ArgumentParseError(e.to_string()))?;
                self.add_cash(args).await
            }
            "add_expense" => {
                let args: AddExpenseArgs = serde_json::from_str(arguments)
                    .map_err(|e| ToolError::ArgumentParseError(e.to_string()))?;
                self.add_expense(args).await
            }
            "modify_expense" => {
                let args: ModifyExpenseArgs = serde_json::from_str(arguments)
                    .map_err(|e| ToolError::ArgumentParseError(e.to_string()))?;
                self.modify_expense(args).await
            }
            "delete_expense" => {
                let args: DeleteExpenseArgs = serde_json::from_str(arguments)
                    .map_err(|e| ToolError::ArgumentParseError(e.to_string()))?;
                self.delete_expense(args).await
            }
            "get_balance" => self.get_balance().await,
            "get_expense_breakdown" => {
                let args: GetExpenseBreakdownArgs = serde_json::from_str(arguments)
                    .map_err(|e| ToolError::ArgumentParseError(e.to_string()))?;
                self.get_expense_breakdown(args).await
            }
            "get_category_expenses" => {
                let args: GetCategoryExpensesArgs = serde_json::from_str(arguments)
                    .map_err(|e| ToolError::ArgumentParseError(e.to_string()))?;
                self.get_category_expenses(args).await
            }
            "get_categories" => self.get_categories().await,
            _ => Err(ToolError::UnknownTool(tool_name.to_string())),
        }
    }

    async fn add_cash(&self, args: AddCashArgs) -> Result<String, ToolError> {
        // TODO: Implement database call
        // self.database.add_cash(args.amount, &args.date).await?;
        todo!("Implement add_cash database method")
    }

    async fn add_expense(&self, args: AddExpenseArgs) -> Result<String, ToolError> {
        // TODO: Implement database call
        // let id = self.database.add_expense(args.amount, &args.description, &args.category, &args.date).await?;
        todo!("Implement add_expense database method")
    }

    async fn modify_expense(&self, args: ModifyExpenseArgs) -> Result<String, ToolError> {
        // TODO: Implement database call
        // self.database.modify_expense(args.expense_id, &args.field, &args.new_value).await?;
        todo!("Implement modify_expense database method")
    }

    async fn delete_expense(&self, args: DeleteExpenseArgs) -> Result<String, ToolError> {
        // TODO: Implement database call
        // self.database.delete_expense(args.expense_id).await?;
        todo!("Implement delete_expense database method")
    }

    async fn get_balance(&self) -> Result<String, ToolError> {
        // TODO: Implement database call
        // let balance = self.database.get_balance().await?;
        todo!("Implement get_balance database method")
    }

    async fn get_expense_breakdown(&self, args: GetExpenseBreakdownArgs) -> Result<String, ToolError> {
        // TODO: Implement database call
        // let breakdown = self.database.get_expense_breakdown(&args.start_date, &args.end_date).await?;
        todo!("Implement get_expense_breakdown database method")
    }

    async fn get_category_expenses(&self, args: GetCategoryExpensesArgs) -> Result<String, ToolError> {
        // TODO: Implement database call
        // let expenses = self.database.get_category_expenses(&args.category, &args.start_date, &args.end_date).await?;
        todo!("Implement get_category_expenses database method")
    }

    async fn get_categories(&self) -> Result<String, ToolError> {
        // TODO: Implement database call
        // let categories = self.database.get_categories().await?;
        todo!("Implement get_categories database method")
    }
}
