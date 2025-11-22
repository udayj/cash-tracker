use super::types::args::*;
use super::visualization;
use crate::{database::DatabaseService, request::SessionContext};
use std::sync::Arc;
use thiserror::Error;
use visualization::generate_pie_chart;

#[derive(Error, Debug)]
pub enum ToolError {
    #[error("Unknown tool: {0}")]
    UnknownTool(String),

    #[error("Failed to parse tool arguments: {0}")]
    ArgumentParseError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Visualization error: {0}")]
    VisualizationError(#[from] visualization::VisualizationError),
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
    ) -> Result<(Option<i64>, String, Option<Vec<u8>>), ToolError> {
        match tool_name {
            "add_cash" => {
                let args: AddCashArgs = serde_json::from_str(arguments)
                    .map_err(|e| ToolError::ArgumentParseError(e.to_string()))?;
                Ok((
                    Some(self.add_cash(&args, ctx).await?),
                    format!("✅ Added ₹{} to cash balance", args.amount),
                    None,
                ))
            }
            "add_expense" => {
                let args: AddExpenseArgs = serde_json::from_str(arguments)
                    .map_err(|e| ToolError::ArgumentParseError(e.to_string()))?;
                Ok((
                    Some(self.add_expense(&args, ctx).await?),
                    format!("✅ Added ₹{} under {}", args.amount, args.category),
                    None,
                ))
            }
            "modify_expense" => {
                let args: ModifyExpenseArgs = serde_json::from_str(arguments)
                    .map_err(|e| ToolError::ArgumentParseError(e.to_string()))?;
                self.modify_expense(args, ctx).await?;
                Ok((None, "✅ Expense modified successfully".to_string(), None))
            }
            "delete_expense" => {
                let args: DeleteExpenseArgs = serde_json::from_str(arguments)
                    .map_err(|e| ToolError::ArgumentParseError(e.to_string()))?;
                self.delete_expense(args, ctx).await?;
                Ok((None, "✅ Expense deleted successfully".to_string(), None))
            }
            "get_balance" => Ok((None, self.get_balance(ctx).await?, None)),
            "get_expense_breakdown" => {
                let args: GetExpenseBreakdownArgs = serde_json::from_str(arguments)
                    .map_err(|e| ToolError::ArgumentParseError(e.to_string()))?;
                self.get_expense_breakdown(args, ctx).await
            }
            "get_category_expenses" => {
                let args: GetCategoryExpensesArgs = serde_json::from_str(arguments)
                    .map_err(|e| ToolError::ArgumentParseError(e.to_string()))?;
                Ok((None, self.get_category_expenses(args, ctx).await?, None))
            }
            "get_categories" => Ok((None, self.get_categories(ctx).await?, None)),
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

    async fn modify_expense(
        &self,
        args: ModifyExpenseArgs,
        ctx: &SessionContext,
    ) -> Result<(), ToolError> {
        self.database
            .modify_expense(args, ctx)
            .await
            .map_err(|e| ToolError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete_expense(
        &self,
        args: DeleteExpenseArgs,
        ctx: &SessionContext,
    ) -> Result<(), ToolError> {
        self.database
            .delete_expense(args.expense_id, ctx)
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
    ) -> Result<(Option<i64>, String, Option<Vec<u8>>), ToolError> {
        let breakdown = self
            .database
            .get_expense_breakdown(ctx.user_id, &args.start_date, &args.end_date)
            .await
            .map_err(|e| ToolError::DatabaseError(e.to_string()))?;

        if breakdown.is_empty() {
            return Ok((None, "No expenses found for this period".to_string(), None));
        }

        let total: i64 = breakdown.iter().map(|s| s.total).sum();
        let mut summary = String::new();

        for category_expense in breakdown.iter() {
            summary.push_str(&format!(
                "{} - Rs.{}\n",
                category_expense.category, category_expense.total
            ));
        }
        summary.push_str(&format!("\nTotal: Rs.{}", total));

        // Generate pie chart with legend
        let chart_data = generate_pie_chart(&breakdown).ok();

        Ok((None, summary, chart_data))
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
