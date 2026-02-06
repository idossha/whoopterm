# whoopterm

A beautiful terminal dashboard for WHOOP fitness data.

![Dashboard Screenshot](docs/screenshot.png)

## Features

- **Terminal UI**: Rich dashboard with charts and tables
- **Real-time Data**: Direct WHOOP API v2 integration
- **Secure**: OAuth2 authentication with local token storage
- **Fast**: Single binary, instant startup
- **Cross-platform**: Works on macOS, Linux, and Windows
- **Comprehensive Metrics**: Recovery scores, sleep analysis, workout tracking
- **Data Visualization**: Charts and graphs for trends over time
- **Keyboard Navigation**: Intuitive controls for dashboard interaction

## Installation

### Homebrew (macOS/Linux)

```bash
brew tap idossha/whoopterm
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
sudo cp target/release/whoopterm /usr/local/bin/
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
- Arrow keys - Navigate
- `Enter` - Select/expand

### Commands

```bash
whoopterm --help           # Show help
whoopterm --auth           # Authenticate with WHOOP
whoopterm --test           # Test API connectivity
whoopterm --refresh        # Force refresh data
whoopterm --version        # Show version
```

## Dashboard Sections

### Today's Metrics
- **Recovery**: Score, resting heart rate, HRV
- **Last Night's Sleep**: Duration, efficiency, sleep stages breakdown
- **Strain**: Daily activity score and target

### Sleep History
- 7-day sleep overview with visual charts
- Hours slept and efficiency percentages
- Sleep consistency tracking

### Recent Workouts
- Last 5 workouts with strain scores
- Duration and average heart rate
- Workout type categorization

### Trends
- Weekly and monthly performance trends
- Recovery pattern analysis
- Sleep quality improvements

## Configuration

whoop-cli reads configuration from environment variables:

| Variable | Description |
|----------|-------------|
| `WHOOP_CLIENT_ID` | Your WHOOP API client ID |
| `WHOOP_CLIENT_SECRET` | Your WHOOP API client secret |
| `WHOOP_CACHE_TTL` | Cache duration in minutes (default: 30) |
| `WHOOP_DATA_DIR` | Custom data directory path |

Add to your shell profile (~/.bashrc, ~/.zshrc, etc.):

```bash
export WHOOP_CLIENT_ID="your_client_id"
export WHOOP_CLIENT_SECRET="your_client_secret"
export WHOOP_CACHE_TTL="60"
```

## Data Storage

All data is stored locally:

- **macOS**: `~/Library/Application Support/whoop-cli/`
- **Linux**: `~/.local/share/whoop-cli/`
- **Windows**: `%APPDATA%/whoop-cli/`

Files:
- `tokens.json` - OAuth tokens
- `cache.json` - Cached fitness data
- `config.json` - User preferences

## Privacy

- All data stored locally on your device
- No data transmitted to third parties
- Direct API connection to WHOOP only
- No analytics or tracking

## Development

### Prerequisites
- Rust 1.70 or later
- Git
- Your favorite editor or IDE

### Building

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run the application
cargo run

# Build for release
cargo build --release
```

### Code Style

- Use `rustfmt` for formatting code
- Use `clippy` for linting
- Follow Rust naming conventions

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy -- -D warnings
```

### Contributing

We welcome contributions! Please read our [Contributing Guidelines](CONTRIBUTING.md) before submitting changes.

## License

MIT License - see [LICENSE](LICENSE)

## Support

- [Issues](https://github.com/idossha/whoopterm/issues)
- [Discussions](https://github.com/idossha/whoopterm/discussions)

## Acknowledgments

- WHOOP for the Developer API
- [ratatui](https://github.com/ratatui-org/ratatui) for the excellent TUI framework
- [reqwest](https://github.com/seanmonstar/reqwest) for HTTP client
- [oauth2](https://github.com/ramosbugs/oauth2-rs) for OAuth2 implementation
