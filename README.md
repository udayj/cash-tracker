# Cash Tracker

A Telegram bot for tracking personal expenses and cash flow using natural language processing.

## Features

- **Natural Language Interface**: Interact with the bot using conversational commands
- **Expense Tracking**: Record expenses with categories, amounts, and dates
- **Cash Management**: Track cash additions and calculate running balance
- **Category Analysis**: View expense breakdowns by category and time period
- **Modification Support**: Edit or delete expenses by replying to bot messages
- **LLM-Powered**: Uses AI to understand and process user requests

## Prerequisites

- Rust (2024 edition)
- Telegram Bot Token
- Turso Database (or compatible libsql instance)

## Environment Variables

Create a `.env` file with the following variables:

```env
TELEGRAM_BOT_TOKEN=your_telegram_bot_token
TURSO_AUTH_TOKEN=your_turso_auth_token
ANTHROPIC_API_KEY=your_anthropic_api_key
```

## Configuration

Edit `config.json` to configure:

```json
{
  "log_level": "info",
  "telegram": {
    "error_channel_id": your_channel_id
  },
  "db_url": "libsql://your-database.turso.io"
}
```

## Installation

```bash
cargo build --release
```

## Usage

```bash
cargo run
```

### Example Commands

Once the bot is running, send messages to your Telegram bot:

- "Add 500 to cash"
- "Spent 250 on groceries"
- "Show my balance"
- "What did I spend on food this month?"
- Reply to an expense message with "Change amount to 300"
- Reply to an expense with "Delete this"

## Database Schema

The application uses two main tables:

- **expenses**: Tracks individual expenses with category, description, and date
- **cash_transactions**: Records cash additions to calculate balance

## Architecture

- **Telegram Bot**: Handles user interactions via Telegram
- **LLM Integration**: Processes natural language requests
- **Database Layer**: Manages persistent storage with Turso/libsql
- **Service Manager**: Orchestrates services with error handling

## License

MIT License - see LICENSE file for details

## Contributing

Contributions are welcome. Please open an issue or submit a pull request.
