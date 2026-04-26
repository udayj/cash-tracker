use super::DatabaseError;
use libsql::Row;

macro_rules! col {
    ($row:expr, $idx:literal) => {
        $row.get($idx)
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?
    };
}

#[derive(Debug, Clone, PartialEq)]
pub enum UserStatus {
    Pending,
    Approved,
    Suspended,
}

impl UserStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserStatus::Pending => "pending",
            UserStatus::Approved => "approved",
            UserStatus::Suspended => "suspended",
        }
    }
}

impl TryFrom<String> for UserStatus {
    type Error = DatabaseError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "pending" => Ok(UserStatus::Pending),
            "approved" => Ok(UserStatus::Approved),
            "suspended" => Ok(UserStatus::Suspended),
            _ => Err(DatabaseError::QueryError(format!(
                "Unknown user status: {}",
                s
            ))),
        }
    }
}

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
            id: col!(row, 0),
            user_id: col!(row, 1),
            amount: col!(row, 2),
            description: col!(row, 3),
            category: col!(row, 4),
            expense_date: col!(row, 5),
            user_message_id: col!(row, 6),
            bot_message_id: col!(row, 7),
            created_at: col!(row, 8),
        })
    }
}

impl CashTransaction {
    pub fn from_row(row: &Row) -> Result<Self, DatabaseError> {
        Ok(Self {
            id: col!(row, 0),
            user_id: col!(row, 1),
            amount: col!(row, 2),
            transaction_date: col!(row, 3),
            user_message_id: col!(row, 4),
            bot_message_id: col!(row, 5),
            created_at: col!(row, 6),
        })
    }
}

impl CategorySummary {
    pub fn from_row(row: &Row) -> Result<Self, DatabaseError> {
        Ok(Self {
            category: col!(row, 0),
            total: col!(row, 1),
        })
    }
}
