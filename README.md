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
- Time-based breakdowns: `expenses this month` or `expenses from 1 oct to 31 oct`
- Category-specific queries: `food expenses this month`
- View all categories: `show categories`
- Visual charts (pie and bar graphs) - TODO

## Architecture

**Stack:**
- Rust application
- Telegram Bot API for messaging
- LLM with function calling (Groq as inference provider)
- Remote Turso database (libsql)
- Chart generation for visualizations - TODO

**Design:**
- Message-based context tracking (no conversation state)
- Immediate commits with easy corrections
- Auto-categorization with user category caching
- Natural language date parsing

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

Ensure all environment variables are properly configured in your production environment.

## License

MIT
