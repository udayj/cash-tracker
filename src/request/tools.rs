use super::types::args::*;
use crate::{database::DatabaseService, request::SessionContext};
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
    ) -> Result<(Option<i64>, String), ToolError> {
        match tool_name {
            "add_cash" => {
                let args: AddCashArgs = serde_json::from_str(arguments)
                    .map_err(|e| ToolError::ArgumentParseError(e.to_string()))?;
                Ok((
                    Some(self.add_cash(&args, ctx).await?),
                    format!("✅ Added ₹{} to cash balance", args.amount),
                ))
            }
            "add_expense" => {
                let args: AddExpenseArgs = serde_json::from_str(arguments)
                    .map_err(|e| ToolError::ArgumentParseError(e.to_string()))?;
                Ok((
                    Some(self.add_expense(&args, ctx).await?),
                    format!("✅ Added ₹{} under {}", args.amount, args.category),
                ))
            }
            "modify_expense" => {
                let args: ModifyExpenseArgs = serde_json::from_str(arguments)
                    .map_err(|e| ToolError::ArgumentParseError(e.to_string()))?;
                self.modify_expense(args).await?;
                Ok((None, "✅ Expense modified successfully".to_string()))
            }
            "delete_expense" => {
                let args: DeleteExpenseArgs = serde_json::from_str(arguments)
                    .map_err(|e| ToolError::ArgumentParseError(e.to_string()))?;
                self.delete_expense(args).await?;
                Ok((None, "✅ Expense deleted successfully".to_string()))
            }
            "get_balance" => Ok((None, self.get_balance(ctx).await?)),
            "get_expense_breakdown" => {
                let args: GetExpenseBreakdownArgs = serde_json::from_str(arguments)
                    .map_err(|e| ToolError::ArgumentParseError(e.to_string()))?;
                Ok((None, self.get_expense_breakdown(args, ctx).await?))
            }
            "get_category_expenses" => {
                let args: GetCategoryExpensesArgs = serde_json::from_str(arguments)
                    .map_err(|e| ToolError::ArgumentParseError(e.to_string()))?;
                Ok((None, self.get_category_expenses(args, ctx).await?))
            }
            "get_categories" => Ok((None, self.get_categories(ctx).await?)),
            _ => Err(ToolError::UnknownTool(tool_name.to_string())),
        }
    }

    async fn add_cash(&self, args: &AddCashArgs, ctx: &SessionContext) -> Result<i64, ToolError> {
        self.database
            .add_cash_transaction(args, ctx)
            .await
            .map_err(|e| ToolError::DatabaseError(e.to_string()))
    }

    async fn add_expense(
        &self,
        args: &AddExpenseArgs,
        ctx: &SessionContext,
    ) -> Result<i64, ToolError> {
        self.database
            .add_expense(args, ctx)
            .await
            .map_err(|e| ToolError::DatabaseError(e.to_string()))
    }

    async fn modify_expense(&self, args: ModifyExpenseArgs) -> Result<(), ToolError> {
        self.database
            .modify_expense(args)
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

    async fn get_balance(&self, ctx: &SessionContext) -> Result<String, ToolError> {
        let balance = self
            .database
            .get_balance(ctx.user_id)
            .await
            .map_err(|e| ToolError::DatabaseError(e.to_string()))?;
        Ok(format!("Cash balance: Rs.{}", balance))
    }

    async fn get_expense_breakdown(
        &self,
        args: GetExpenseBreakdownArgs,
        ctx: &SessionContext,
    ) -> Result<String, ToolError> {
        let breakdown = self
            .database
            .get_expense_breakdown(ctx.user_id, &args.start_date, &args.end_date)
            .await
            .map_err(|e| ToolError::DatabaseError(e.to_string()))?;
        let mut summary: String = "".to_string();
        for category_expense in breakdown {
            summary.push_str(
                format!(
                    "{} - Rs.{}\n",
                    category_expense.category, category_expense.total
                )
                .as_str(),
            );
        }
        Ok(summary)
    }

    async fn get_category_expenses(
        &self,
        args: GetCategoryExpensesArgs,
        ctx: &SessionContext,
    ) -> Result<String, ToolError> {
        let expenses = self
            .database
            .get_category_expenses(
                ctx.user_id,
                &args.category,
                &args.start_date,
                &args.end_date,
            )
            .await
            .map_err(|e| ToolError::DatabaseError(e.to_string()))?;

        let mut summary: String = "".to_string();
        for expense in expenses {
            summary.push_str(&expense.description);
            summary.push('\n');
        }
        Ok(summary)
    }

    async fn get_categories(&self, ctx: &SessionContext) -> Result<String, ToolError> {
        let categories = self
            .database
            .get_categories(ctx.user_id)
            .await
            .map_err(|e| ToolError::DatabaseError(e.to_string()))?;

        // Categories will be formatted in request/mod.rs format_response
        Ok(categories.join("\n"))
    }
}
