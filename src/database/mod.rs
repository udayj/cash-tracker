use libsql::{Builder, Connection, params, params::IntoParams};
use std::{env, time::Duration};
use thiserror::Error;

use crate::{
    core::ExpirableCache,
    request::types::{
        SessionContext,
        args::{AddCashArgs, AddExpenseArgs, ModifyExpenseArgs},
    },
};
mod types;

pub use types::*;

#[derive(Error, Debug, Clone)]
pub enum DatabaseError {
    #[error("Error building init database:{0}")]
    DatabaseBuildError(String),

    #[error("Database connection error: {0}")]
    ConnectionError(String),

    #[error("Database query error: {0}")]
    QueryError(String),
}

const DEFAULT_CATEGORY_CACHE_TTL: u64 = 86400 * 30;
pub struct DatabaseService {
    pub conn: Connection,
    pub category_cache: ExpirableCache<i64, Vec<String>>,
}

impl DatabaseService {
    pub async fn new(db_url: String) -> Result<Self, DatabaseError> {
        let url = db_url;
        let token = env::var("TURSO_AUTH_TOKEN").expect("TURSO_AUTH_TOKEN must be set");

        let db = Builder::new_remote(url, token)
            .build()
            .await
            .map_err(|e| DatabaseError::DatabaseBuildError(e.to_string()))?;
        let conn = db
            .connect()
            .map_err(|e| DatabaseError::ConnectionError(e.to_string()))?;
        let category_cache =
            ExpirableCache::new(10, Duration::from_secs(DEFAULT_CATEGORY_CACHE_TTL));
        Ok(Self {
            conn,
            category_cache,
        })
    }
}

impl DatabaseService {
    async fn execute_returning_id(
        &self,
        sql: &str,
        params: impl IntoParams,
    ) -> Result<i64, DatabaseError> {
        self.conn
            .execute(sql, params)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }

    async fn execute(&self, sql: &str, params: impl IntoParams) -> Result<(), DatabaseError> {
        self.conn
            .execute(sql, params)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(())
    }

    pub async fn add_expense(
        &self,
        args: &AddExpenseArgs,
        session_context: &SessionContext,
    ) -> Result<i64, DatabaseError> {
        // Update cache if current category is a new category
        if let Some(mut cache) = self.category_cache.get(&session_context.user_id)
            && !cache.contains(&args.category)
        {
            cache.push(args.category.clone());
            self.category_cache.remove(&session_context.user_id);
            self.category_cache.insert(session_context.user_id, cache);
        }

        self.execute_returning_id(
            "INSERT INTO expenses (user_id, amount, description, category, expense_date, user_message_id, created_at)
             VALUES (?, ?, ?, ?, ?, ?, datetime('now'))", 
            params![session_context.user_id, args.amount, args.description.to_string(), args.category.to_string(), args.date.to_string(), session_context.user_message_id]
        ).await
    }

    pub async fn update_expense_bot_message(
        &self,
        expense_id: i64,
        bot_message_id: i64,
    ) -> Result<(), DatabaseError> {
        self.execute(
            "UPDATE expenses SET bot_message_id = ? WHERE id = ?",
            params![bot_message_id, expense_id],
        )
        .await
    }

    // Add cash transaction, returns transaction_id
    pub async fn add_cash_transaction(
        &self,
        args: &AddCashArgs,
        session_context: &SessionContext,
    ) -> Result<i64, DatabaseError> {
        self.execute_returning_id(
            "INSERT INTO cash_transactions (user_id, amount, transaction_date, user_message_id, created_at)
             VALUES (?, ?, ?, ?, datetime('now'))",
            params![session_context.user_id, args.amount, args.date.to_string(), session_context.user_message_id],
        )
        .await
    }

    // Update cash transaction with bot_message_id
    pub async fn update_cash_bot_message(
        &self,
        cash_id: i64,
        bot_message_id: i64,
    ) -> Result<(), DatabaseError> {
        self.execute(
            "UPDATE cash_transactions SET bot_message_id = ? WHERE id = ?",
            params![bot_message_id, cash_id],
        )
        .await
    }

    // Modify expense fields
    pub async fn modify_expense(
        &self,
        args: ModifyExpenseArgs,
        ctx: &SessionContext,
    ) -> Result<(), DatabaseError> {
        // Update cache if there is a category and it is a new category
        if let Some(mut cache) = self.category_cache.get(&ctx.user_id)
            && let Some(category) = &args.category
            && !cache.contains(category)
        {
            cache.push(category.clone());
            self.category_cache.remove(&ctx.user_id);
            self.category_cache.insert(ctx.user_id, cache);
        }

        let mut set_clauses = Vec::new();
        let mut values: Vec<libsql::Value> = Vec::new();

        if let Some(amt) = args.amount {
            set_clauses.push("amount = ?");
            values.push(amt.into());
        }
        if let Some(desc) = args.description {
            set_clauses.push("description = ?");
            values.push(desc.into());
        }
        if let Some(cat) = args.category {
            set_clauses.push("category = ?");
            values.push(cat.into());
        }
        if let Some(d) = args.date {
            set_clauses.push("expense_date = ?");
            values.push(d.into());
        }

        if set_clauses.is_empty() {
            return Err(DatabaseError::QueryError("No fields to update".to_string()));
        }

        let sql = format!(
            "UPDATE expenses SET {} WHERE id = ? AND user_id = ?",
            set_clauses.join(", ")
        );
        values.push(args.expense_id.into());
        values.push(ctx.user_id.into());
        self.execute(&sql, libsql::params::Params::Positional(values))
            .await
    }

    // Delete expense
    pub async fn delete_expense(
        &self,
        expense_id: i64,
        ctx: &SessionContext,
    ) -> Result<(), DatabaseError> {
        self.execute(
            "DELETE FROM expenses WHERE id = ? AND user_id = ?",
            params![expense_id, ctx.user_id],
        )
        .await
    }

    // Get balance (cash added - expenses)
    pub async fn get_balance(&self, user_id: i64) -> Result<i64, DatabaseError> {
        let stmt = self
            .conn
            .prepare(
                "SELECT
                    (SELECT COALESCE(SUM(amount), 0) FROM cash_transactions WHERE user_id = ?) -
                    (SELECT COALESCE(SUM(amount), 0) FROM expenses WHERE user_id = ?)
                 AS balance",
            )
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let mut rows = stmt
            .query(params![user_id, user_id])
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?
        {
            let balance: i64 = row
                .get(0)
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
            Ok(balance)
        } else {
            Ok(0)
        }
    }

    // Get expense breakdown by category for date range
    pub async fn get_expense_breakdown(
        &self,
        user_id: i64,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<CategorySummary>, DatabaseError> {
        let stmt = self
            .conn
            .prepare(
                "SELECT category, SUM(amount) as total
                 FROM expenses
                 WHERE user_id = ? AND expense_date BETWEEN ? AND ?
                 GROUP BY category
                 ORDER BY total DESC",
            )
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let mut rows = stmt
            .query(params![user_id, start_date, end_date])
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let mut summaries = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?
        {
            summaries.push(CategorySummary::from_row(&row)?);
        }
        Ok(summaries)
    }

    // Get expenses for specific category and date range
    pub async fn get_category_expenses(
        &self,
        user_id: i64,
        category: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<Expense>, DatabaseError> {
        let stmt = self
            .conn
            .prepare(
                "SELECT id, user_id, amount, description, category, expense_date, user_message_id, bot_message_id, created_at
                 FROM expenses
                 WHERE user_id = ? AND category = ? AND expense_date BETWEEN ? AND ?
                 ORDER BY expense_date DESC",
            )
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let mut rows = stmt
            .query(params![user_id, category, start_date, end_date])
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let mut expenses = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?
        {
            expenses.push(Expense::from_row(&row)?);
        }
        Ok(expenses)
    }

    // Get all categories for user
    pub async fn get_categories(&self, user_id: i64) -> Result<Vec<String>, DatabaseError> {
        // If categories for the user are available in the cache, use it
        if let Some(cache) = self.category_cache.get(&user_id) {
            return Ok(cache);
        }

        let stmt = self
            .conn
            .prepare("SELECT DISTINCT category FROM expenses WHERE user_id = ? ORDER BY category")
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let mut rows = stmt
            .query(params![user_id])
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let mut categories = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?
        {
            let category: String = row
                .get(0)
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
            categories.push(category);
        }
        // Update cache
        self.category_cache.insert(user_id, categories.clone());
        Ok(categories)
    }

    // Find expense by message ID (for reply-based modifications)
    pub async fn find_expense_by_message(
        &self,
        user_id: i64,
        message_id: i64,
    ) -> Result<Option<Expense>, DatabaseError> {
        let stmt = self
            .conn
            .prepare(
                "SELECT id, user_id, amount, description, category, expense_date, user_message_id, bot_message_id, created_at
                 FROM expenses
                 WHERE user_id = ? AND (user_message_id = ? OR bot_message_id = ?)",
            )
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let mut rows = stmt
            .query(params![user_id, message_id, message_id])
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?
        {
            Ok(Some(Expense::from_row(&row)?))
        } else {
            Ok(None)
        }
    }

    // Find cash transaction by message ID
    pub async fn find_cash_by_message(
        &self,
        user_id: i64,
        message_id: i64,
    ) -> Result<Option<CashTransaction>, DatabaseError> {
        let stmt = self
            .conn
            .prepare(
                "SELECT id, user_id, amount, transaction_date, user_message_id, bot_message_id, created_at
                 FROM cash_transactions
                 WHERE user_id = ? AND (user_message_id = ? OR bot_message_id = ?)",
            )
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let mut rows = stmt
            .query(params![user_id, message_id, message_id])
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?
        {
            Ok(Some(CashTransaction::from_row(&row)?))
        } else {
            Ok(None)
        }
    }
}
