# whoopterm

A beautiful terminal dashboard for WHOOP fitness data.

![Dashboard Screenshot](docs/screenshot.png)

## Features

- **Terminal UI**: Rich dashboard with charts and tables
- **Real-time Data**: Direct WHOOP API v2 integration
- **Secure**: OAuth2 authentication with local token storage
- **Fast**: Single binary, instant startup
- **Cross-platform**: Works on macOS, Linux, and Windows

## Installation

### Homebrew (macOS/Linux)

```bash
brew tap idossha/whoop
brew install whoopterm
```

### APT (Debian/Ubuntu)

```bash
echo "deb [trusted=yes] https://apt.idossha.dev/whoop stable main" | sudo tee /etc/apt/sources.list.d/whoop.list
sudo apt update
sudo apt install whoopterm
```

### From Source

Requires: Rust 1.70+

```bash
git clone https://github.com/idossha/whoopterm.git
cd whoopterm
cargo build --release
sudo cp target/release/whoop /usr/local/bin/
```

## Usage

### First Time Setup

1. Get API credentials from [WHOOP Developer Dashboard](https://developer-dashboard.whoop.com)
2. Set environment variables:

```bash
export WHOOP_CLIENT_ID="your_client_id"
export WHOOP_CLIENT_SECRET="your_client_secret"
```

3. Authenticate:

```bash
whoop --auth
```

### Dashboard

Launch the dashboard:

```bash
whoop
```

**Controls:**
- `r` - Refresh data
- `q` or `Esc` - Quit

### Commands

```bash
whoop --help           # Show help
whoop --auth           # Authenticate with WHOOP
whoop --test           # Test API connectivity
whoop --refresh        # Force refresh data
```

## Dashboard Sections

### Today's Metrics
- **Recovery**: Score, resting heart rate, HRV
- **Last Night's Sleep**: Duration, efficiency, sleep stages breakdown

### Sleep History
- 7-day sleep overview with visual charts
- Hours slept and efficiency percentages

### Recent Workouts
- Last 5 workouts with strain scores
- Duration and average heart rate

## Configuration

whoop-cli reads configuration from environment variables:

| Variable | Description |
|----------|-------------|
| `WHOOP_CLIENT_ID` | Your WHOOP API client ID |
| `WHOOP_CLIENT_SECRET` | Your WHOOP API client secret |

Add to your shell profile (~/.bashrc, ~/.zshrc, etc.):

```bash
export WHOOP_CLIENT_ID="your_client_id"
export WHOOP_CLIENT_SECRET="your_client_secret"
```

## Data Storage

All data is stored locally:

- **macOS**: `~/Library/Application Support/whoop-cli/`
- **Linux**: `~/.local/share/whoop-cli/`
- **Windows**: `%APPDATA%/whoop-cli/`

Files:
- `tokens.json` - OAuth tokens
- `cache.json` - Cached fitness data

## Privacy

- All data stored locally on your device
- No data transmitted to third parties
- Direct API connection to WHOOP only

## License

MIT License - see [LICENSE](LICENSE)

## Contributing

Contributions welcome! Please submit issues and pull requests.

## Acknowledgments

- WHOOP for the Developer API
- [ratatui](https://github.com/ratatui-org/ratatui) for the excellent TUI framework
