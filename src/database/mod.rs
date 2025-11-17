use libsql::{params, Builder, Connection, Row};
use std::env;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum DatabaseError {
    #[error("Error building init database:{0}")]
    DatabaseBuildError(String),

    #[error("Database connection error: {0}")]
    ConnectionError(String),

    #[error("Database query error: {0}")]
    QueryError(String),
}

// Entity structs
#[derive(Debug, Clone)]
pub struct Expense {
    pub id: i64,
    pub user_id: i64,
    pub amount: i64,
    pub description: String,
    pub category: String,
    pub expense_date: String,
    pub user_message_id: i64,
    pub bot_message_id: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct CashTransaction {
    pub id: i64,
    pub user_id: i64,
    pub amount: i64,
    pub transaction_date: String,
    pub user_message_id: i64,
    pub bot_message_id: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct CategorySummary {
    pub category: String,
    pub total: i64,
}

pub struct DatabaseService {
    pub conn: Connection,
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
        Ok(Self { conn })
    }

    // Helper: execute INSERT/UPDATE and return last_insert_rowid
    async fn execute_returning_id(
        &self,
        sql: &str,
        params: impl libsql::params::IntoParams,
    ) -> Result<i64, DatabaseError> {
        self.conn
            .execute(sql, params)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }

    // Helper: execute INSERT/UPDATE without returning ID
    async fn execute(
        &self,
        sql: &str,
        params: impl libsql::params::IntoParams,
    ) -> Result<(), DatabaseError> {
        self.conn
            .execute(sql, params)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        Ok(())
    }

    // Add expense, returns expense_id
    pub async fn add_expense(
        &self,
        user_id: i64,
        amount: i64,
        description: &str,
        category: &str,
        expense_date: &str,
        user_message_id: i64,
    ) -> Result<i64, DatabaseError> {
        self.execute_returning_id(
            "INSERT INTO expenses (user_id, amount, description, category, expense_date, user_message_id, created_at)
             VALUES (?, ?, ?, ?, ?, ?, datetime('now'))",
            params![user_id, amount, description, category, expense_date, user_message_id],
        ).await
    }

    // Update expense with bot_message_id
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
        user_id: i64,
        amount: i64,
        transaction_date: &str,
        user_message_id: i64,
    ) -> Result<i64, DatabaseError> {
        self.execute_returning_id(
            "INSERT INTO cash_transactions (user_id, amount, transaction_date, user_message_id, created_at)
             VALUES (?, ?, ?, ?, datetime('now'))",
            params![user_id, amount, transaction_date, user_message_id],
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

    // Modify expense field
    pub async fn modify_expense(
        &self,
        expense_id: i64,
        field: &str,
        new_value: &str,
    ) -> Result<(), DatabaseError> {
        // Validate field to prevent SQL injection
        let sql = match field {
            "amount" => {
                let amount: i64 = new_value
                    .parse()
                    .map_err(|_| DatabaseError::QueryError("Invalid amount".to_string()))?;
                self.execute(
                    "UPDATE expenses SET amount = ? WHERE id = ?",
                    params![amount, expense_id],
                )
                .await?;
                return Ok(());
            }
            "description" => "UPDATE expenses SET description = ? WHERE id = ?",
            "category" => "UPDATE expenses SET category = ? WHERE id = ?",
            "expense_date" => "UPDATE expenses SET expense_date = ? WHERE id = ?",
            _ => {
                return Err(DatabaseError::QueryError(format!(
                    "Invalid field: {}",
                    field
                )))
            }
        };

        self.execute(sql, params![new_value, expense_id]).await
    }

    // Delete expense
    pub async fn delete_expense(&self, expense_id: i64) -> Result<(), DatabaseError> {
        self.execute("DELETE FROM expenses WHERE id = ?", params![expense_id])
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
                 AS balance"
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
        let stmt = self
            .conn
            .prepare(
                "SELECT DISTINCT category FROM expenses WHERE user_id = ? ORDER BY category",
            )
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

// Row conversion implementations
impl Expense {
    fn from_row(row: &Row) -> Result<Self, DatabaseError> {
        Ok(Self {
            id: row
                .get(0)
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?,
            user_id: row
                .get(1)
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?,
            amount: row
                .get(2)
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?,
            description: row
                .get(3)
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?,
            category: row
                .get(4)
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?,
            expense_date: row
                .get(5)
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?,
            user_message_id: row
                .get(6)
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?,
            bot_message_id: row
                .get(7)
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?,
            created_at: row
                .get(8)
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?,
        })
    }
}

impl CashTransaction {
    fn from_row(row: &Row) -> Result<Self, DatabaseError> {
        Ok(Self {
            id: row
                .get(0)
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?,
            user_id: row
                .get(1)
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?,
            amount: row
                .get(2)
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?,
            transaction_date: row
                .get(3)
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?,
            user_message_id: row
                .get(4)
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?,
            bot_message_id: row
                .get(5)
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?,
            created_at: row
                .get(6)
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?,
        })
    }
}

impl CategorySummary {
    fn from_row(row: &Row) -> Result<Self, DatabaseError> {
        Ok(Self {
            category: row
                .get(0)
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?,
            total: row
                .get(1)
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?,
        })
    }
}
