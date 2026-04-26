pub const INSERT_EXPENSE: &str =
    "INSERT INTO expenses (user_id, amount, description, category, expense_date, user_message_id, created_at)
     VALUES (?, ?, ?, ?, ?, ?, datetime('now'))";

pub const UPDATE_EXPENSE_BOT_MESSAGE: &str = "UPDATE expenses SET bot_message_id = ? WHERE id = ?";

pub const DELETE_EXPENSE: &str = "DELETE FROM expenses WHERE id = ? AND user_id = ?";

pub const INSERT_CASH_TRANSACTION: &str =
    "INSERT INTO cash_transactions (user_id, amount, transaction_date, user_message_id, created_at)
     VALUES (?, ?, ?, ?, datetime('now'))";

pub const UPDATE_CASH_BOT_MESSAGE: &str =
    "UPDATE cash_transactions SET bot_message_id = ? WHERE id = ?";

pub const GET_BALANCE: &str = "SELECT
        (SELECT COALESCE(SUM(amount), 0) FROM cash_transactions WHERE user_id = ?) -
        (SELECT COALESCE(SUM(amount), 0) FROM expenses WHERE user_id = ?)
     AS balance";

pub const GET_EXPENSE_BREAKDOWN: &str = "SELECT category, SUM(amount) as total
     FROM expenses
     WHERE user_id = ? AND expense_date BETWEEN ? AND ?
     GROUP BY category
     ORDER BY total DESC";

pub const GET_CATEGORY_EXPENSES: &str =
    "SELECT id, user_id, amount, description, category, expense_date, user_message_id, bot_message_id, created_at
     FROM expenses
     WHERE user_id = ? AND category = ? AND expense_date BETWEEN ? AND ?
     ORDER BY expense_date DESC";

pub const GET_CATEGORIES: &str =
    "SELECT DISTINCT category FROM expenses WHERE user_id = ? ORDER BY category";

pub const FIND_EXPENSE_BY_MESSAGE: &str =
    "SELECT id, user_id, amount, description, category, expense_date, user_message_id, bot_message_id, created_at
     FROM expenses
     WHERE user_id = ? AND (user_message_id = ? OR bot_message_id = ?)";

pub const FIND_CASH_BY_MESSAGE: &str =
    "SELECT id, user_id, amount, transaction_date, user_message_id, bot_message_id, created_at
     FROM cash_transactions
     WHERE user_id = ? AND (user_message_id = ? OR bot_message_id = ?)";

pub const GET_USER_STATUS: &str = "SELECT status FROM allowlist WHERE user_id = ?";

pub const INSERT_PENDING_USER: &str =
    "INSERT OR IGNORE INTO allowlist (user_id, status) VALUES (?, 'pending')";

pub const UPDATE_USER_STATUS: &str = "UPDATE allowlist SET status = ? WHERE user_id = ?";
