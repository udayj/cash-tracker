use super::DatabaseError;
use libsql::Row;

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

impl Expense {
    pub fn from_row(row: &Row) -> Result<Self, DatabaseError> {
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
    pub fn from_row(row: &Row) -> Result<Self, DatabaseError> {
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
    pub fn from_row(row: &Row) -> Result<Self, DatabaseError> {
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
