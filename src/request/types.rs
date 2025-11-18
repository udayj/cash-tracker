use crate::database::{CashTransaction, Expense};

#[derive(Debug, Clone)]
pub enum ActionType {
    Expense,
    CashTransaction,
}

#[derive(Debug, Clone)]
pub struct FinalizeAction {
    pub record_id: i64,
    pub action_type: ActionType,
}

pub struct FulfilmentResult {
    pub response: String,
    pub finalize: Option<FinalizeAction>,
}

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

pub mod args {

    use serde::Deserialize;

    // Tool argument structs
    #[derive(Debug, Deserialize, Clone)]
    pub struct AddCashArgs {
        pub amount: f64,
        pub date: String,
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct AddExpenseArgs {
        pub amount: f64,
        pub description: String,
        pub category: String,
        pub date: String,
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct ModifyExpenseArgs {
        pub expense_id: i64,
        pub amount: Option<i64>,
        pub description: Option<String>,
        pub category: Option<String>,
        pub date: Option<String>,
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
}
