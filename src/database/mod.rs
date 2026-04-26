use libsql::{Builder, Connection, Database, params, params::IntoParams};
use moka::sync::Cache;
use std::{env, time::Duration};
use thiserror::Error;

use crate::request::types::{
    SessionContext,
    args::{AddCashArgs, AddExpenseArgs, ModifyExpenseArgs},
};
mod queries;
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
    pub db: Database,
    pub category_cache: Cache<i64, Vec<String>>,
}

impl DatabaseService {
    pub async fn new(db_url: String) -> Result<Self, DatabaseError> {
        let token = env::var("TURSO_AUTH_TOKEN").expect("TURSO_AUTH_TOKEN must be set");

        let db = Builder::new_remote(db_url, token)
            .build()
            .await
            .map_err(|e| DatabaseError::DatabaseBuildError(e.to_string()))?;
        let category_cache = Cache::builder()
            .max_capacity(10)
            .time_to_live(Duration::from_secs(DEFAULT_CATEGORY_CACHE_TTL))
            .build();
        Ok(Self { db, category_cache })
    }

    async fn get_connection(&self) -> Result<Connection, DatabaseError> {
        self.db
            .connect()
            .map_err(|e| DatabaseError::ConnectionError(e.to_string()))
    }
}

impl DatabaseService {
    async fn execute_returning_id(
        &self,
        sql: &str,
        params: impl IntoParams,
    ) -> Result<i64, DatabaseError> {
        let conn = self.get_connection().await?;
        conn.execute(sql, params)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        Ok(conn.last_insert_rowid())
    }

    async fn execute(&self, sql: &str, params: impl IntoParams) -> Result<(), DatabaseError> {
        let conn = self.get_connection().await?;
        conn.execute(sql, params)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        Ok(())
    }

    pub async fn add_expense(
        &self,
        args: &AddExpenseArgs,
        session_context: &SessionContext,
    ) -> Result<i64, DatabaseError> {
        if let Some(mut cache) = self.category_cache.get(&session_context.user_id)
            && !cache.contains(&args.category)
        {
            cache.push(args.category.clone());
            self.category_cache.insert(session_context.user_id, cache);
        }

        self.execute_returning_id(
            queries::INSERT_EXPENSE,
            params![
                session_context.user_id,
                args.amount,
                args.description.to_string(),
                args.category.to_string(),
                args.date.to_string(),
                session_context.user_message_id
            ],
        )
        .await
    }

    pub async fn update_expense_bot_message(
        &self,
        expense_id: i64,
        bot_message_id: i64,
    ) -> Result<(), DatabaseError> {
        self.execute(
            queries::UPDATE_EXPENSE_BOT_MESSAGE,
            params![bot_message_id, expense_id],
        )
        .await
    }

    pub async fn add_cash_transaction(
        &self,
        args: &AddCashArgs,
        session_context: &SessionContext,
    ) -> Result<i64, DatabaseError> {
        self.execute_returning_id(
            queries::INSERT_CASH_TRANSACTION,
            params![
                session_context.user_id,
                args.amount,
                args.date.to_string(),
                session_context.user_message_id
            ],
        )
        .await
    }

    pub async fn update_cash_bot_message(
        &self,
        cash_id: i64,
        bot_message_id: i64,
    ) -> Result<(), DatabaseError> {
        self.execute(
            queries::UPDATE_CASH_BOT_MESSAGE,
            params![bot_message_id, cash_id],
        )
        .await
    }

    pub async fn modify_expense(
        &self,
        args: ModifyExpenseArgs,
        ctx: &SessionContext,
    ) -> Result<(), DatabaseError> {
        if let Some(mut cache) = self.category_cache.get(&ctx.user_id)
            && let Some(category) = &args.category
            && !cache.contains(category)
        {
            cache.push(category.clone());
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

    pub async fn delete_expense(
        &self,
        expense_id: i64,
        ctx: &SessionContext,
    ) -> Result<(), DatabaseError> {
        self.execute(queries::DELETE_EXPENSE, params![expense_id, ctx.user_id])
            .await
    }

    pub async fn get_balance(&self, user_id: i64) -> Result<i64, DatabaseError> {
        let conn = self.get_connection().await?;
        let stmt = conn
            .prepare(queries::GET_BALANCE)
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
            row.get(0)
                .map_err(|e| DatabaseError::QueryError(e.to_string()))
        } else {
            Ok(0)
        }
    }

    pub async fn get_expense_breakdown(
        &self,
        user_id: i64,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<CategorySummary>, DatabaseError> {
        let conn = self.get_connection().await?;
        let stmt = conn
            .prepare(queries::GET_EXPENSE_BREAKDOWN)
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

    pub async fn get_category_expenses(
        &self,
        user_id: i64,
        category: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<Expense>, DatabaseError> {
        let conn = self.get_connection().await?;
        let stmt = conn
            .prepare(queries::GET_CATEGORY_EXPENSES)
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

    pub async fn get_categories(&self, user_id: i64) -> Result<Vec<String>, DatabaseError> {
        if let Some(cache) = self.category_cache.get(&user_id) {
            return Ok(cache);
        }

        let conn = self.get_connection().await?;
        let stmt = conn
            .prepare(queries::GET_CATEGORIES)
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
        self.category_cache.insert(user_id, categories.clone());
        Ok(categories)
    }

    pub async fn find_expense_by_message(
        &self,
        user_id: i64,
        message_id: i64,
    ) -> Result<Option<Expense>, DatabaseError> {
        let conn = self.get_connection().await?;
        let stmt = conn
            .prepare(queries::FIND_EXPENSE_BY_MESSAGE)
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

    pub async fn find_cash_by_message(
        &self,
        user_id: i64,
        message_id: i64,
    ) -> Result<Option<CashTransaction>, DatabaseError> {
        let conn = self.get_connection().await?;
        let stmt = conn
            .prepare(queries::FIND_CASH_BY_MESSAGE)
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

    pub async fn get_user_status(&self, user_id: i64) -> Result<Option<UserStatus>, DatabaseError> {
        let conn = self.get_connection().await?;
        let stmt = conn
            .prepare(queries::GET_USER_STATUS)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let mut rows = stmt
            .query(params![user_id])
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?
        {
            let status: String = row
                .get(0)
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
            Ok(Some(UserStatus::try_from(status)?))
        } else {
            Ok(None)
        }
    }

    pub async fn insert_pending_user(&self, user_id: i64) -> Result<(), DatabaseError> {
        self.execute(queries::INSERT_PENDING_USER, params![user_id])
            .await
    }

    pub async fn set_user_status(
        &self,
        user_id: i64,
        status: UserStatus,
    ) -> Result<bool, DatabaseError> {
        let conn = self.get_connection().await?;
        let rows_affected = conn
            .execute(
                queries::UPDATE_USER_STATUS,
                params![status.as_str(), user_id],
            )
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        Ok(rows_affected > 0)
    }
}
