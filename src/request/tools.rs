use crate::database::{DatabaseService, Expense, CashTransaction};
use serde::Deserialize;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum RecordContext {
    Expense(Expense),
    CashTransaction(CashTransaction),
}

#[derive(Debug, Clone)]
pub struct SessionContext {
    pub user_id: i64,
    pub user_message_id: i64,
    pub replied_record: Option<RecordContext>,
}

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
        ctx: &SessionContext,
    ) -> Result<Option<i64>, ToolError> {
        match tool_name {
            "add_cash" => {
                let args: AddCashArgs = serde_json::from_str(arguments)
                    .map_err(|e| ToolError::ArgumentParseError(e.to_string()))?;
                self.add_cash(args, ctx).await.map(Some)
            }
            "add_expense" => {
                let args: AddExpenseArgs = serde_json::from_str(arguments)
                    .map_err(|e| ToolError::ArgumentParseError(e.to_string()))?;
                self.add_expense(args, ctx).await.map(Some)
            }
            "modify_expense" => {
                let args: ModifyExpenseArgs = serde_json::from_str(arguments)
                    .map_err(|e| ToolError::ArgumentParseError(e.to_string()))?;
                self.modify_expense(args).await.map(|_| None)
            }
            "delete_expense" => {
                let args: DeleteExpenseArgs = serde_json::from_str(arguments)
                    .map_err(|e| ToolError::ArgumentParseError(e.to_string()))?;
                self.delete_expense(args).await.map(|_| None)
            }
            "get_balance" => self.get_balance(ctx).await.map(|_| None),
            "get_expense_breakdown" => {
                let args: GetExpenseBreakdownArgs = serde_json::from_str(arguments)
                    .map_err(|e| ToolError::ArgumentParseError(e.to_string()))?;
                self.get_expense_breakdown(args, ctx).await.map(|_| None)
            }
            "get_category_expenses" => {
                let args: GetCategoryExpensesArgs = serde_json::from_str(arguments)
                    .map_err(|e| ToolError::ArgumentParseError(e.to_string()))?;
                self.get_category_expenses(args, ctx).await.map(|_| None)
            }
            "get_categories" => self.get_categories(ctx).await.map(|_| None),
            _ => Err(ToolError::UnknownTool(tool_name.to_string())),
        }
    }

    async fn add_cash(&self, args: AddCashArgs, ctx: &SessionContext) -> Result<i64, ToolError> {
        // Convert rupees to paisa (multiply by 100)
        let amount_paisa = (args.amount * 100.0) as i64;

        let id = self
            .database
            .add_cash_transaction(
                ctx.user_id,
                amount_paisa,
                &args.date,
                ctx.user_message_id,
            )
            .await
            .map_err(|e| ToolError::DatabaseError(e.to_string()))?;

        Ok(id)
    }

    async fn add_expense(&self, args: AddExpenseArgs, ctx: &SessionContext) -> Result<i64, ToolError> {
        // Convert rupees to paisa (multiply by 100)
        let amount_paisa = (args.amount * 100.0) as i64;

        let id = self
            .database
            .add_expense(
                ctx.user_id,
                amount_paisa,
                &args.description,
                &args.category,
                &args.date,
                ctx.user_message_id,
            )
            .await
            .map_err(|e| ToolError::DatabaseError(e.to_string()))?;

        Ok(id)
    }

    async fn modify_expense(&self, args: ModifyExpenseArgs) -> Result<(), ToolError> {
        self.database
            .modify_expense(args.expense_id, &args.field, &args.new_value)
            .await
            .map_err(|e| ToolError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete_expense(&self, args: DeleteExpenseArgs) -> Result<(), ToolError> {
        self.database
            .delete_expense(args.expense_id)
            .await
            .map_err(|e| ToolError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get_balance(&self, ctx: &SessionContext) -> Result<(), ToolError> {
        let _balance = self
            .database
            .get_balance(ctx.user_id)
            .await
            .map_err(|e| ToolError::DatabaseError(e.to_string()))?;

        // Balance will be formatted in request/mod.rs format_response
        Ok(())
    }

    async fn get_expense_breakdown(&self, args: GetExpenseBreakdownArgs, ctx: &SessionContext) -> Result<(), ToolError> {
        let _breakdown = self
            .database
            .get_expense_breakdown(ctx.user_id, &args.start_date, &args.end_date)
            .await
            .map_err(|e| ToolError::DatabaseError(e.to_string()))?;

        // Breakdown will be formatted in request/mod.rs format_response
        Ok(())
    }

    async fn get_category_expenses(&self, args: GetCategoryExpensesArgs, ctx: &SessionContext) -> Result<(), ToolError> {
        let _expenses = self
            .database
            .get_category_expenses(ctx.user_id, &args.category, &args.start_date, &args.end_date)
            .await
            .map_err(|e| ToolError::DatabaseError(e.to_string()))?;

        // Expenses will be formatted in request/mod.rs format_response
        Ok(())
    }

    async fn get_categories(&self, ctx: &SessionContext) -> Result<(), ToolError> {
        let _categories = self
            .database
            .get_categories(ctx.user_id)
            .await
            .map_err(|e| ToolError::DatabaseError(e.to_string()))?;

        // Categories will be formatted in request/mod.rs format_response
        Ok(())
    }
}
