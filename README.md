# Algorithms Trading

A comprehensive Rust-based trading toolkit built on the Alpaca Trading API. This project provides multiple binaries for streaming market data, analyzing historical data, and performing market analysis with support for various output formats and data feeds.

## Features

- 🚀 **Real-time market data streaming** with WebSocket support
- 📊 **Historical market data retrieval** with multiple timeframes
- 📈 **Data analysis tools** for captured market data
- 💾 **Multiple output formats**: Plain text, JSON, CSV
- 🔄 **Multiple data feeds**: SIP, IEX, BOATS, OTC
- 📁 **File and console output** with append support
- 🧪 **Comprehensive test suite** (44+ tests)
- ⚡ **Async/await support** with Tokio runtime

## Project Structure

This project uses a library + multiple binaries architecture:

```
src/
├── lib.rs              # Shared library with core functionality
├── main.rs             # streaming-client binary
├── historical.rs       # historical-data binary
└── analyzer.rs         # data-analyzer binary
tests/
├── integration_tests.rs  # CLI integration tests
└── mock_tests.rs       # API mock tests
```

## Binaries

### 1. streaming-client
Real-time market data streaming with WebSocket connection and automatic reconnection.

```bash
cargo run --bin streaming-client -- [OPTIONS]
```

**Options:**
- `-o, --output <OUTPUT>`: Output to file instead of console
- `-a, --append`: Append to existing file instead of overwriting
- `-f, --format <FORMAT>`: Data format (plain, json, csv) [default: plain]

### 2. historical-data
Retrieve historical bar data for specified symbols and date ranges.

```bash
cargo run --bin historical-data -- --symbols <SYMBOLS> --start <START> --end <END> [OPTIONS]
```

**Options:**
- `-s, --symbols <SYMBOLS>`: Symbols to retrieve (comma-separated)
- `--start <START>`: Start date (YYYY-MM-DD)
- `--end <END>`: End date (YYYY-MM-DD)
- `-t, --timeframe <TIMEFRAME>`: Bar timeframe [default: 1Day]
- `-o, --output <OUTPUT>`: Output file (optional)
- `-f, --format <FORMAT>`: Output format (plain, json, csv) [default: plain]
- `-a, --append`: Append to existing file
- `--page-size <PAGE_SIZE>`: Request page size (max 10000) [default: 1000]
- `--feed <FEED>`: Data feed source (sip, iex, boats, otc) [default: sip]

### 3. data-analyzer
Analyze captured streaming data files and generate statistics.

```bash
cargo run --bin data-analyzer -- --input <INPUT> [OPTIONS]
```

**Options:**
- `-i, --input <INPUT>`: Input file to analyze
- `-f, --format <FORMAT>`: Input format (plain, json, csv) [default: json]

## Installation

### Prerequisites
- Rust 1.70+ 
- Cargo
- Git

### Setup

1. **Clone the repository:**
   ```bash
   git clone <repository-url>
   cd algorithms-trading
   ```

2. **Set up environment variables:**
   Create a `.env` file in the project root:
   ```env
   APCA_API_KEY_ID=your_alpaca_key_id
   APCA_API_SECRET_KEY=your_alpaca_secret_key
   APCA_API_BASE_URL=https://paper-api.alpaca.markets  # for paper trading
   ```

3. **Build the project:**
   ```bash
   cargo build --release
   ```

4. **Run tests:**
   ```bash
   cargo test
   ```

## Usage Examples

### Streaming Market Data

**Stream to console (plain text):**
```bash
cargo run --bin streaming-client
```

**Stream to JSON file:**
```bash
cargo run --bin streaming-client -- --output market_data.json --format json
```

**Append to existing CSV file:**
```bash
cargo run --bin streaming-client -- --output data.csv --format csv --append
```

### Historical Data Retrieval

**Get daily bars for AAPL:**
```bash
cargo run --bin historical-data -- --symbols AAPL --start 2024-01-01 --end 2024-01-31
```

**Get minute bars for multiple symbols with IEX feed:**
```bash
cargo run --bin historical-data -- \
  --symbols AAPL,MSFT,GOOGL \
  --start 2024-01-01 \
  --end 2024-01-02 \
  --timeframe 1Min \
  --feed iex \
  --format json \
  --output historical_data.json
```

**Get overnight trading data:**
```bash
cargo run --bin historical-data -- \
  --symbols AAPL \
  --start 2024-01-01 \
  --end 2024-01-02 \
  --feed boats \
  --format csv \
  --output overnight_data.csv
```

### Data Analysis

**Analyze captured JSON data:**
```bash
cargo run --bin data-analyzer -- --input market_data.json --format json
```

**Analyze CSV data:**
```bash
cargo run --bin data-analyzer -- --input data.csv --format csv
```

## Supported Timeframes

- `1Min`, `5Min`, `15Min`, `30Min` - Intraday bars
- `1Hour` - Hourly bars  
- `1Day` - Daily bars
- `1Week` - Weekly bars
- `1Month` - Monthly bars

## Data Feeds

- **SIP** (default): All US exchanges - comprehensive market data
- **IEX**: Investors Exchange - commission-free trading data
- **BOATS**: Blue Ocean ATS - overnight US trading data
- **OTC**: Over-the-counter exchanges - OTC market data

## Output Formats

### Plain Text
Human-readable format with price changes and percentages:
```
📊 AAPL: 2024-01-15T10:00:00Z | O: $150.00 H: $155.00 L: $148.00 C: $153.00 | Vol: 10000 | Change: $3.00 (2.00%)
```

### JSON
Structured JSON format for programmatic processing:
```json
{"symbol":"AAPL","timestamp":"2024-01-15T10:00:00Z","open":150.0,"high":155.0,"low":148.0,"close":153.0,"volume":10000,"trade_count":500,"vwap":151.5}
```

### CSV  
Spreadsheet-compatible format with headers:
```csv
symbol,timestamp,open,high,low,close,volume,trade_count,vwap
AAPL,2024-01-15T10:00:00Z,150.00,155.00,148.00,153.00,10000,500,151.5
```

## Configuration

### Environment Variables
- `APCA_API_KEY_ID`: Alpaca API key ID
- `APCA_API_SECRET_KEY`: Alpaca API secret key  
- `APCA_API_BASE_URL`: API base URL (paper or live trading)

### Logging
Set log level using the `RUST_LOG` environment variable:
```bash
export RUST_LOG=info
cargo run --bin streaming-client
```

## Development

### Running Tests
```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --bin historical-data        # Unit tests
cargo test --test integration_tests     # Integration tests  
cargo test --test mock_tests           # Mock tests
```

### Adding New Features
1. Update the shared library in `src/lib.rs` for common functionality
2. Add binary-specific code in the appropriate `src/*.rs` file
3. Add comprehensive tests in `tests/` or module test sections
4. Update this README with new features

### Dependencies
- **alpaca-trading-api-rust**: Alpaca Trading API client
- **tokio**: Async runtime
- **clap**: Command-line argument parsing
- **serde**: Serialization/deserialization
- **chrono**: Date/time handling
- **anyhow**: Error handling
- **dotenv**: Environment variable loading

## Testing

The project includes a comprehensive test suite:

- **18 unit tests**: Core functionality testing
- **14 integration tests**: CLI argument validation and error handling  
- **12 mock tests**: API interaction simulation

### Test Coverage
- Date parsing and validation
- Timeframe validation
- Data feed validation
- Output format testing
- File I/O operations
- Error scenarios
- CLI argument parsing

## Troubleshooting

### Common Issues

**Build errors:**
```bash
# Update dependencies
cargo update

# Clean and rebuild
cargo clean
cargo build
```

**API connection issues:**
- Verify your `.env` file has correct API credentials
- Check if using paper trading URL for testing
- Ensure network connectivity

**Permission errors:**
- Check file write permissions for output directories
- Verify the output path exists

### Getting Help
- Check the help for any binary: `cargo run --bin <binary-name> -- --help`
- Review error messages for specific guidance
- Ensure environment variables are properly set

## License

This project is for educational and research purposes. Please ensure compliance with Alpaca's terms of service and applicable financial regulations.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass: `cargo test`
5. Submit a pull request

## Changelog

### Latest Version
- ✅ Updated to latest alpaca-trading-api-rust
- ✅ Added data feed selection (SIP, IEX, BOATS, OTC)
- ✅ Enhanced error handling and validation
- ✅ Comprehensive test suite (44+ tests)
- ✅ Multiple output formats (Plain, JSON, CSV)
- ✅ File and console output with append support

---

**Built with ❤️ and Rust** 🦀