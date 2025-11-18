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

## Deployment

### Docker Deployment

The project includes Docker support for easy deployment to any VPS or cloud provider.

#### Building the Docker Image

```bash
docker build -t cash-tracker .
```

#### Running with Docker Compose

1. Copy the environment template:
```bash
cp .env.example .env
```

2. Edit `.env` with your actual credentials

3. Start the service:
```bash
docker compose up -d
```

### Automated Deployment to Hetzner VPS

The project uses GitHub Actions for CI/CD. On every push to `main`:

1. CI tests run (build, test, clippy, format)
2. Docker image is built and pushed to GitHub Container Registry
3. VPS automatically pulls and deploys the new version

#### VPS Setup

**One-time setup on your Hetzner VPS:**

```bash
# Install Docker and Docker Compose
curl -fsSL https://get.docker.com -o get-docker.sh
sh get-docker.sh
apt-get install docker-compose-plugin

# Create deployment directory
mkdir -p /opt/cash-tracker
cd /opt/cash-tracker

# Create .env file with your secrets
nano .env

# Download docker-compose.yml from the repository
wget https://raw.githubusercontent.com/udayj/cash-tracker/main/docker-compose.yml

# Log in to GitHub Container Registry (if private repo)
docker login ghcr.io -u YOUR_GITHUB_USERNAME

# Start the service
docker compose up -d
```

#### GitHub Secrets Configuration

Add these secrets to your GitHub repository (Settings → Secrets and variables → Actions):

- `VPS_HOST`: Your Hetzner VPS IP address
- `VPS_USER`: SSH username (usually `root`)
- `VPS_SSH_KEY`: Private SSH key for authentication

The deployment workflow will automatically deploy on push to main.

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
