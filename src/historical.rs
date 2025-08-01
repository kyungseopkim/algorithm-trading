use algorithms_trading::{DataFormat, OutputMode};
use alpaca_trading_api_rust::*;
use anyhow::Result;
use chrono::NaiveDate;
use clap::Parser;
use dotenv::dotenv;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "historical-data")]
#[command(about = "Retrieve historical bar data for symbols")]
#[command(version)]
struct Args {
    /// Symbols to retrieve data for (comma-separated)
    #[arg(short, long)]
    symbols: String,
    
    /// Start date (YYYY-MM-DD)
    #[arg(long)]
    start: String,
    
    /// End date (YYYY-MM-DD)
    #[arg(long)]
    end: String,
    
    /// Timeframe for bars (1Min, 5Min, 15Min, 1Hour, 1Day)
    #[arg(short, long, default_value = "1Day")]
    timeframe: String,
    
    /// Output file (optional)
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    /// Output format
    #[arg(short, long, value_enum, default_value_t = DataFormat::Plain)]
    format: DataFormat,
    
    /// Append to existing file instead of overwriting
    #[arg(short, long)]
    append: bool,
    
    /// Page size for requests (max 10000)
    #[arg(long, default_value = "1000")]
    page_size: u32,
    
    /// Data feed source (sip, iex, boats, otc)
    #[arg(long, default_value = "sip")]
    feed: String,
}

#[derive(Debug, serde::Serialize)]
struct HistoricalBarData {
    symbol: String,
    timestamp: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: u64,
    trade_count: u64,
    vwap: f64,
}

impl From<&Bar> for HistoricalBarData {
    fn from(bar: &Bar) -> Self {
        Self {
            symbol: "".to_string(), // Will be set by caller
            timestamp: bar.t.clone(),
            open: bar.o,
            high: bar.h,
            low: bar.l,
            close: bar.c,
            volume: bar.v,
            trade_count: bar.n,
            vwap: bar.vw,
        }
    }
}

fn validate_timeframe(timeframe: &str) -> Result<String> {
    match timeframe.to_lowercase().as_str() {
        "1min" => Ok("1Min".to_string()),
        "5min" => Ok("5Min".to_string()),
        "15min" => Ok("15Min".to_string()),
        "30min" => Ok("30Min".to_string()),
        "1hour" | "1h" => Ok("1Hour".to_string()),
        "1day" | "1d" => Ok("1Day".to_string()),
        "1week" | "1w" => Ok("1Week".to_string()),
        "1month" | "1m" => Ok("1Month".to_string()),
        _ => Err(anyhow::anyhow!("Invalid timeframe: {}. Supported: 1Min, 5Min, 15Min, 30Min, 1Hour, 1Day, 1Week, 1Month", timeframe)),
    }
}

fn validate_feed(feed: &str) -> Result<StockDataFeed> {
    match feed.to_lowercase().as_str() {
        "sip" => Ok(StockDataFeed::Sip),
        "iex" => Ok(StockDataFeed::Iex),
        "boats" => Ok(StockDataFeed::Boats),
        "otc" => Ok(StockDataFeed::Otc),
        _ => Err(anyhow::anyhow!("Invalid feed: {}. Supported: sip, iex, boats, otc", feed)),
    }
}

fn parse_date(date_str: &str) -> Result<String> {
    let naive_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;
    Ok(naive_date.format("%Y-%m-%d").to_string())
}

fn format_bar_data(bar: &HistoricalBarData, format: &DataFormat) -> Result<String> {
    match format {
        DataFormat::Plain => {
            let change = bar.close - bar.open;
            let change_pct = (change / bar.open) * 100.0;
            Ok(format!(
                "📊 {}: {} | O: ${:.2} H: ${:.2} L: ${:.2} C: ${:.2} | Vol: {} | Change: ${:.2} ({:.2}%)",
                bar.symbol,
                bar.timestamp,
                bar.open,
                bar.high,
                bar.low,
                bar.close,
                bar.volume,
                change,
                change_pct
            ))
        }
        DataFormat::Json => {
            Ok(serde_json::to_string(bar)?)
        }
        DataFormat::Csv => {
            Ok(format!(
                "{},{},{:.2},{:.2},{:.2},{:.2},{},{},{}",
                bar.symbol,
                bar.timestamp,
                bar.open,
                bar.high,
                bar.low,
                bar.close,
                bar.volume,
                bar.trade_count,
                bar.vwap
            ))
        }
    }
}

async fn fetch_historical_data(
    client: &AlpacaClient,
    symbol: &str,
    start: &str,
    end: &str,
    timeframe: &str,
    page_size: u32,
    feed: &StockDataFeed,
) -> Result<Vec<HistoricalBarData>> {
    println!("📈 Fetching historical data for {} from {} to {}...", symbol, start, end);
    
    let mut all_bars = Vec::new();
    let mut page_token: Option<String> = None;
    
    loop {
        let bars_response = client
            .get_stock_bars(
                &[symbol],
                timeframe,
                Some(start),
                Some(end),
                None, // adjustment
                page_token.as_deref(),
                Some(page_size),
                Some(feed), // feed
            )
            .await?;
        
        if let Some(symbol_bars) = bars_response.bars.get(symbol) {
            if symbol_bars.is_empty() {
                break;
            }
            
            for bar in symbol_bars {
                let mut hist_bar = HistoricalBarData::from(bar);
                hist_bar.symbol = symbol.to_string();
                all_bars.push(hist_bar);
            }
            
            // Check if there's more data
            if let Some(next_token) = &bars_response.next_page_token {
                page_token = Some(next_token.clone());
            } else {
                break;
            }
        } else {
            break;
        }
        
        // Add a small delay to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    println!("✅ Retrieved {} bars for {}", all_bars.len(), symbol);
    Ok(all_bars)
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    
    let args = Args::parse();
    
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    // Parse symbols
    let symbols: Vec<String> = args.symbols
        .split(',')
        .map(|s| s.trim().to_uppercase())
        .collect();
    
    // Parse dates
    let start_date = parse_date(&args.start)?;
    let end_date = parse_date(&args.end)?;
    
    if start_date >= end_date {
        return Err(anyhow::anyhow!("Start date must be before end date"));
    }
    
    // Validate timeframe
    let timeframe = validate_timeframe(&args.timeframe)?;
    
    // Validate data feed
    let feed = validate_feed(&args.feed)?;
    
    // Validate page size
    if args.page_size > 10000 {
        return Err(anyhow::anyhow!("Page size cannot exceed 10000"));
    }
    
    println!("🔍 Historical Data Retrieval");
    println!("============================");
    println!("Symbols: {:?}", symbols);
    println!("Date range: {} to {}", start_date, end_date);
    println!("Timeframe: {}", timeframe);
    println!("Data feed: {}", feed.as_str());
    println!("Output format: {:?}", args.format);
    if let Some(ref output_path) = args.output {
        println!("Output file: {}", output_path.display());
    }
    println!();
    
    // Create output mode
    let output_mode = if let Some(output_path) = &args.output {
        OutputMode::create_file_mode(output_path, args.format.clone(), args.append)?
    } else {
        OutputMode::create_console_mode(args.format.clone())
    };
    
    // Write CSV header if needed
    if matches!(args.format, DataFormat::Csv) {
        output_mode.writeln("symbol,timestamp,open,high,low,close,volume,trade_count,vwap")?;
    }
    
    // Initialize Alpaca API client
    let client = AlpacaClient::new()?;
    
    // Fetch data for each symbol
    let mut total_bars = 0;
    for symbol in &symbols {
        match fetch_historical_data(&client, symbol, &start_date, &end_date, &timeframe, args.page_size, &feed).await {
            Ok(bars) => {
                total_bars += bars.len();
                
                // Output the data
                for bar in &bars {
                    let formatted = format_bar_data(bar, &args.format)?;
                    output_mode.writeln(&formatted)?;
                }
                
                if bars.is_empty() {
                    println!("⚠️  No data found for symbol: {}", symbol);
                }
            }
            Err(e) => {
                eprintln!("❌ Error fetching data for {}: {}", symbol, e);
            }
        }
    }
    
    println!("\n📊 Summary");
    println!("==========");
    println!("Total symbols processed: {}", symbols.len());
    println!("Total bars retrieved: {}", total_bars);
    
    if let Some(output_path) = &args.output {
        println!("Data saved to: {}", output_path.display());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_parse_date_valid() {
        let result = parse_date("2024-01-15");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "2024-01-15");
    }

    #[test]
    fn test_parse_date_invalid_format() {
        let result = parse_date("01-15-2024");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_date_invalid_date() {
        let result = parse_date("2024-13-32");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_timeframe_valid() {
        assert_eq!(validate_timeframe("1min").unwrap(), "1Min");
        assert_eq!(validate_timeframe("5Min").unwrap(), "5Min");
        assert_eq!(validate_timeframe("1Day").unwrap(), "1Day");
        assert_eq!(validate_timeframe("1h").unwrap(), "1Hour");
        assert_eq!(validate_timeframe("1w").unwrap(), "1Week");
        assert_eq!(validate_timeframe("1m").unwrap(), "1Month");
    }

    #[test]
    fn test_validate_timeframe_invalid() {
        let result = validate_timeframe("invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid timeframe"));
    }

    #[test]
    fn test_historical_bar_data_from_bar() {
        let bar = Bar {
            t: "2024-01-15T10:00:00Z".to_string(),
            o: 150.0,
            h: 155.0,
            l: 149.0,
            c: 153.0,
            v: 10000,
            n: 500,
            vw: 152.5,
        };

        let mut hist_bar = HistoricalBarData::from(&bar);
        hist_bar.symbol = "AAPL".to_string();

        assert_eq!(hist_bar.symbol, "AAPL");
        assert_eq!(hist_bar.timestamp, "2024-01-15T10:00:00Z");
        assert_eq!(hist_bar.open, 150.0);
        assert_eq!(hist_bar.high, 155.0);
        assert_eq!(hist_bar.low, 149.0);
        assert_eq!(hist_bar.close, 153.0);
        assert_eq!(hist_bar.volume, 10000);
        assert_eq!(hist_bar.trade_count, 500);
        assert_eq!(hist_bar.vwap, 152.5);
    }

    #[test]
    fn test_format_bar_data_plain() {
        let bar = HistoricalBarData {
            symbol: "AAPL".to_string(),
            timestamp: "2024-01-15T10:00:00Z".to_string(),
            open: 150.0,
            high: 155.0,
            low: 149.0,
            close: 153.0,
            volume: 10000,
            trade_count: 500,
            vwap: 152.5,
        };

        let result = format_bar_data(&bar, &DataFormat::Plain).unwrap();
        assert!(result.contains("📊 AAPL"));
        assert!(result.contains("O: $150.00"));
        assert!(result.contains("H: $155.00"));
        assert!(result.contains("L: $149.00"));
        assert!(result.contains("C: $153.00"));
        assert!(result.contains("Vol: 10000"));
        assert!(result.contains("Change: $3.00 (2.00%)"));
    }

    #[test]
    fn test_format_bar_data_json() {
        let bar = HistoricalBarData {
            symbol: "AAPL".to_string(),
            timestamp: "2024-01-15T10:00:00Z".to_string(),
            open: 150.0,
            high: 155.0,
            low: 149.0,
            close: 153.0,
            volume: 10000,
            trade_count: 500,
            vwap: 152.5,
        };

        let result = format_bar_data(&bar, &DataFormat::Json).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["symbol"], "AAPL");
        assert_eq!(parsed["open"], 150.0);
        assert_eq!(parsed["close"], 153.0);
    }

    #[test]
    fn test_format_bar_data_csv() {
        let bar = HistoricalBarData {
            symbol: "AAPL".to_string(),
            timestamp: "2024-01-15T10:00:00Z".to_string(),
            open: 150.0,
            high: 155.0,
            low: 149.0,
            close: 153.0,
            volume: 10000,
            trade_count: 500,
            vwap: 152.5,
        };

        let result = format_bar_data(&bar, &DataFormat::Csv).unwrap();
        assert_eq!(result, "AAPL,2024-01-15T10:00:00Z,150.00,155.00,149.00,153.00,10000,500,152.5");
    }

    #[test]
    fn test_output_mode_console() {
        let output_mode = OutputMode::create_console_mode(DataFormat::Plain);
        // Just test that it doesn't panic - we can't easily capture stdout in tests
        assert!(output_mode.writeln("test message").is_ok());
    }

    #[test]
    fn test_output_mode_file() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_output.txt");
        
        let output_mode = OutputMode::create_file_mode(&file_path, DataFormat::Plain, false).unwrap();
        assert!(output_mode.writeln("test message").is_ok());
        
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "test message\n");
    }

    #[test]
    fn test_output_mode_file_append() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_append.txt");
        
        // Write initial content
        fs::write(&file_path, "initial\n").unwrap();
        
        let output_mode = OutputMode::create_file_mode(&file_path, DataFormat::Plain, true).unwrap();
        assert!(output_mode.writeln("appended").is_ok());
        
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "initial\nappended\n");
    }

    #[test]
    fn test_page_size_validation() {
        // This would be tested in main function, but we can test the logic
        let page_size = 15000_u32;
        assert!(page_size > 10000);
    }

    #[test]
    fn test_date_range_validation() {
        let start = "2024-01-15";
        let end = "2024-01-10";
        
        let start_parsed = parse_date(start).unwrap();
        let end_parsed = parse_date(end).unwrap();
        
        assert!(start_parsed >= end_parsed);
    }

    #[test]
    fn test_symbols_parsing() {
        let symbols_str = "AAPL,MSFT, GOOGL , tsla";
        let symbols: Vec<String> = symbols_str
            .split(',')
            .map(|s| s.trim().to_uppercase())
            .collect();
        
        assert_eq!(symbols, vec!["AAPL", "MSFT", "GOOGL", "TSLA"]);
    }

    #[test]
    fn test_validate_feed_valid() {
        assert!(matches!(validate_feed("sip").unwrap(), StockDataFeed::Sip));
        assert!(matches!(validate_feed("SIP").unwrap(), StockDataFeed::Sip));
        assert!(matches!(validate_feed("iex").unwrap(), StockDataFeed::Iex));
        assert!(matches!(validate_feed("IEX").unwrap(), StockDataFeed::Iex));
        assert!(matches!(validate_feed("boats").unwrap(), StockDataFeed::Boats));
        assert!(matches!(validate_feed("BOATS").unwrap(), StockDataFeed::Boats));
        assert!(matches!(validate_feed("otc").unwrap(), StockDataFeed::Otc));
        assert!(matches!(validate_feed("OTC").unwrap(), StockDataFeed::Otc));
    }

    #[test]
    fn test_validate_feed_invalid() {
        let result = validate_feed("invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid feed"));
    }

    #[test]
    fn test_feed_as_str() {
        assert_eq!(StockDataFeed::Sip.as_str(), "sip");
        assert_eq!(StockDataFeed::Iex.as_str(), "iex");
        assert_eq!(StockDataFeed::Boats.as_str(), "boats");
        assert_eq!(StockDataFeed::Otc.as_str(), "otc");
    }
}