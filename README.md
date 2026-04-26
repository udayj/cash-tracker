# Cash Tracker

A minimalist personal expense tracking bot that uses Telegram as the interface and LLM-powered natural language understanding.

## Features

### Cash Management
- Add or subtract cash: `add cash 500` or `-200 cash`
- Specify dates: `add cash 1000 on 12.8.25` (defaults to today)
- Check balance: `current balance`

### Expense Tracking
- Add expenses naturally: `500 fruits` or `fruits 500`
- Automatic categorization via LLM with confirmation
- Specify dates: `500 batteries on 15.10.25`
- Modify or delete by replying to any message: `change to 400`, `delete`, or `change category to Food`

### Expense Analysis
- Time-based breakdowns: `expenses this month` or `expenses from 1 oct to 31 oct` — includes pie chart
- Category-specific queries: `food expenses this month`
- View all categories: `show categories`

## Architecture

**Stack:**
- Rust application
- Telegram Bot API for messaging
- LLM with function calling (Groq as inference provider)
- Remote Turso database (libsql)
- Pie chart generation via plotters

**Design:**
- Message-based context tracking (no conversation state)
- Immediate commits with easy corrections
- Auto-categorization with user category caching
- Natural language date parsing
- Allowlist-based access control

## Setup

### Prerequisites
- Rust 1.89.0+
- Telegram bot token (via [@BotFather](https://t.me/botfather))
- Turso database
- Groq API key

### Environment Variables

Create a `.env` file:

```env
TELEGRAM_BOT_TOKEN=your_bot_token
ERROR_BOT_TOKEN=your_error_bot_token
TURSO_AUTH_TOKEN=your_turso_token
GROQ_API_KEY=your_groq_key
TELEGRAM_ERROR_CHANNEL_ID=your_error_channel_id
```

### Configuration

The app reads `config.json` at startup (override with `CONFIG_FILE` env var):

```json
{
    "log_level": "info",
    "db_url": "libsql://your-db.turso.io",
    "model_name": "openai/gpt-oss-20b",
    "admin_id": 123456789
}
```

`admin_id` is your Telegram user ID — message [@userinfobot](https://t.me/userinfobot) to find it. Use `config.prod.json` for production.

### Database

Create the following table in your Turso database (in addition to the `expenses` and `cash_transactions` tables):

```sql
CREATE TABLE allowlist (user_id INTEGER PRIMARY KEY, status TEXT NOT NULL DEFAULT 'pending');
```

### Access Control

The bot uses an allowlist. By default no one has access until approved by the admin.

- When an unknown user messages the bot, they are silently queued as `pending` and you receive a notification in your error channel with their user ID
- From your Telegram account, send `/approve <user_id>` to the bot to grant access
- Send `/suspend <user_id>` to revoke access
- As admin, you bypass the allowlist entirely

### Local Development

```bash
# Install dependencies
cargo build

# Run the bot
cargo run
```

## Deployment

### Docker

Build and run:

```bash
docker build -t cash-tracker .
docker run -d --env-file .env cash-tracker
```

### Production

The application is designed to run on any VPS:

```bash
# Build release binary
cargo build --release

# Run with environment variables
./target/release/cash-tracker
```

Ensure all environment variables and `config.prod.json` are properly configured in your production environment.

## License

MIT
