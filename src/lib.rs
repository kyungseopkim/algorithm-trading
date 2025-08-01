use alpaca_trading_api_rust::*;
use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::ValueEnum;
use csv::Writer;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::timeout;

#[derive(Debug, Clone, ValueEnum)]
pub enum DataFormat {
    /// Plain text format (default)
    Plain,
    /// JSON format for structured data
    Json,
    /// CSV format for spreadsheet compatibility
    Csv,
}

impl Default for DataFormat {
    fn default() -> Self {
        DataFormat::Plain
    }
}

#[derive(Debug, Clone)]
pub enum OutputMode {
    Console { format: DataFormat },
    File { 
        file: Arc<Mutex<std::fs::File>>, 
        format: DataFormat,
        csv_writer: Option<Arc<Mutex<Writer<std::fs::File>>>>,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StreamingData {
    pub timestamp: DateTime<Utc>,
    pub message_type: String,
    pub symbol: Option<String>,
    pub data: serde_json::Value,
}

impl OutputMode {
    pub fn write(&self, message: &str) -> Result<()> {
        match self {
            OutputMode::Console { .. } => {
                print!("{}", message);
                std::io::stdout().flush()?;
            }
            OutputMode::File { file, .. } => {
                let mut file = file.lock().unwrap();
                write!(file, "{}", message)?;
                file.flush()?;
            }
        }
        Ok(())
    }
    
    pub fn writeln(&self, message: &str) -> Result<()> {
        self.write(&format!("{}\n", message))
    }
    
    pub fn write_streaming_data(&self, data: &StreamingData) -> Result<()> {
        match self {
            OutputMode::Console { format } => {
                match format {
                    DataFormat::Plain => {
                        self.writeln(&self.format_plain(data))?;
                    }
                    DataFormat::Json => {
                        self.writeln(&serde_json::to_string(data)?)?;
                    }
                    DataFormat::Csv => {
                        self.writeln(&self.format_csv_line(data))?;
                    }
                }
            }
            OutputMode::File { file, format, csv_writer } => {
                match format {
                    DataFormat::Plain => {
                        let mut file = file.lock().unwrap();
                        writeln!(file, "{}", self.format_plain(data))?;
                        file.flush()?;
                    }
                    DataFormat::Json => {
                        let mut file = file.lock().unwrap();
                        writeln!(file, "{}", serde_json::to_string(data)?)?;
                        file.flush()?;
                    }
                    DataFormat::Csv => {
                        if let Some(csv_writer) = csv_writer {
                            let mut writer = csv_writer.lock().unwrap();
                            self.write_csv_record(&mut writer, data)?;
                            writer.flush()?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
    
    fn format_plain(&self, data: &StreamingData) -> String {
        match data.message_type.as_str() {
            "t" => {
                if let Ok(trade) = serde_json::from_value::<StreamingTrade>(data.data.clone()) {
                    format!("🔄 Trade: {} - ${:.2} x {} @ {} (Exchange: {}, ID: {})", 
                        trade.symbol, trade.price, trade.size, 
                        trade.timestamp, trade.exchange, trade.id)
                } else {
                    format!("🔄 Trade: {}", data.data)
                }
            }
            "q" => {
                if let Ok(quote) = serde_json::from_value::<StreamingQuote>(data.data.clone()) {
                    let spread = quote.ask_price - quote.bid_price;
                    format!("💰 Quote: {} - Bid: ${:.2} x {} | Ask: ${:.2} x {} | Spread: ${:.2} @ {}", 
                        quote.symbol, quote.bid_price, quote.bid_size, 
                        quote.ask_price, quote.ask_size, spread, quote.timestamp)
                } else {
                    format!("💰 Quote: {}", data.data)
                }
            }
            "b" => {
                if let Ok(bar) = serde_json::from_value::<StreamingBar>(data.data.clone()) {
                    let change = bar.close - bar.open;
                    let change_pct = (change / bar.open) * 100.0;
                    format!("📈 Bar: {} - O: ${:.2} H: ${:.2} L: ${:.2} C: ${:.2} V: {} | Change: ${:.2} ({:.2}%) @ {}", 
                        bar.symbol, bar.open, bar.high, bar.low, 
                        bar.close, bar.volume, change, change_pct, bar.timestamp)
                } else {
                    format!("📈 Bar: {}", data.data)
                }
            }
            "success" => format!("✅ Success: {}", data.data),
            "subscription" => format!("📡 Subscription: {}", data.data),
            "error" => format!("❌ Error: {}", data.data),
            _ => format!("❓ Unknown: {} - {}", data.message_type, data.data),
        }
    }
    
    fn format_csv_line(&self, data: &StreamingData) -> String {
        format!("{},{},{},{}", 
            data.timestamp.format("%Y-%m-%d %H:%M:%S%.3f UTC"),
            data.message_type,
            data.symbol.as_deref().unwrap_or(""),
            data.data.to_string().replace(",", ";"))
    }
    
    fn write_csv_record<W: Write>(&self, writer: &mut Writer<W>, data: &StreamingData) -> Result<()> {
        writer.write_record(&[
            data.timestamp.format("%Y-%m-%d %H:%M:%S%.3f UTC").to_string(),
            data.message_type.clone(),
            data.symbol.as_deref().unwrap_or("").to_string(),
            data.data.to_string(),
        ])?;
        Ok(())
    }
    
    pub fn create_file_mode(output_path: &PathBuf, format: DataFormat, append: bool) -> Result<Self> {
        let file = if append {
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(output_path)?
        } else {
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(output_path)?
        };
        
        // Create CSV writer if format is CSV
        let csv_writer = if matches!(format, DataFormat::Csv) {
            let csv_file = if append {
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(output_path)?
            } else {
                OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(output_path)?
            };
            
            let mut writer = Writer::from_writer(csv_file);
            
            // Write CSV header if not appending
            if !append {
                writer.write_record(&["timestamp", "message_type", "symbol", "data"])?;
                writer.flush()?;
            }
            
            Some(Arc::new(Mutex::new(writer)))
        } else {
            None
        };
        
        Ok(OutputMode::File { 
            file: Arc::new(Mutex::new(file)),
            format,
            csv_writer,
        })
    }
    
    pub fn create_console_mode(format: DataFormat) -> Self {
        OutputMode::Console { format }
    }
}

#[derive(Debug)]
pub struct StreamingConfig {
    pub feed: StreamingFeed,
    pub trade_symbols: Vec<String>,
    pub quote_symbols: Vec<String>,
    pub bar_symbols: Vec<String>,
    pub max_retries: u32,
    pub auth_timeout: Duration,
    pub subscribe_timeout: Duration,
    pub output_mode: OutputMode,
}

impl StreamingConfig {
    pub fn new(output_mode: OutputMode) -> Self {
        let feed = match std::env::var("ALPACA_FEED") {
            Ok(f) if f.to_lowercase() == "sip" => StreamingFeed::Sip,
            Ok(f) if f.to_lowercase() == "delayed_sip" => StreamingFeed::DelayedSip,
            _ => StreamingFeed::Iex,
        };
        
        Self {
            feed,
            trade_symbols: get_symbols_from_env("TRADE_SYMBOLS", vec!["AAPL", "GOOGL", "TSLA", "MSFT"]),
            quote_symbols: get_symbols_from_env("QUOTE_SYMBOLS", vec!["AAPL", "MSFT", "NVDA"]),
            bar_symbols: get_symbols_from_env("BAR_SYMBOLS", vec!["AAPL", "SPY"]),
            max_retries: 5,
            auth_timeout: Duration::from_secs(10),
            subscribe_timeout: Duration::from_secs(10),
            output_mode,
        }
    }
}

pub async fn run_streaming_client(config: &StreamingConfig) -> Result<()> {
    config.output_mode.writeln(&format!("📡 Using streaming feed: {:?}", config.feed))?;
    
    let streaming_client = StreamingClient::new(config.feed.clone())?;
    let mut connection = streaming_client.connect().await?;
    
    config.output_mode.writeln("🔐 Authenticating...")?;
    timeout(config.auth_timeout, connection.authenticate()).await??;
    config.output_mode.writeln("✅ Authentication successful")?;
    
    let mut subscriptions = HashMap::new();
    subscriptions.insert(StreamingDataType::Trades, config.trade_symbols.clone());
    subscriptions.insert(StreamingDataType::Quotes, config.quote_symbols.clone());
    subscriptions.insert(StreamingDataType::Bars, config.bar_symbols.clone());
    
    config.output_mode.writeln("📋 Subscribing to data streams...")?;
    timeout(config.subscribe_timeout, connection.subscribe(subscriptions)).await??;
    
    config.output_mode.writeln("✅ Successfully subscribed to:")?;
    config.output_mode.writeln(&format!("  📊 Trades: {:?}", config.trade_symbols))?;
    config.output_mode.writeln(&format!("  💰 Quotes: {:?}", config.quote_symbols))?;
    config.output_mode.writeln(&format!("  📈 Bars: {:?}", config.bar_symbols))?;
    config.output_mode.writeln("\nPress Ctrl+C to exit gracefully...\n")?;
    
    use tokio::signal;
    let ctrl_c = signal::ctrl_c();
    tokio::pin!(ctrl_c);
    
    let output_mode = config.output_mode.clone();
    tokio::select! {
        result = connection.run(move |message| {
            process_streaming_message(&message, &output_mode)
        }) => {
            if let Err(e) = result {
                eprintln!("❌ Streaming connection error: {}", e);
                return Err(e);
            }
        }
        _ = &mut ctrl_c => {
            config.output_mode.writeln("\n🛑 Received interrupt signal, shutting down gracefully...")?;
        }
    }
    config.output_mode.writeln("👋 Advanced streaming example terminated.")?;
    Ok(())
}

pub fn process_streaming_message(message: &StreamingMessage, output_mode: &OutputMode) -> Result<()> {
    match message.message_type.as_str() {
        "t" => handle_trade_message(message, output_mode),
        "q" => handle_quote_message(message, output_mode),
        "b" => handle_bar_message(message, output_mode),
        "success" => handle_success_message(message, output_mode),
        "subscription" => handle_subscription_message(message, output_mode),
        "error" => handle_error_message(message, output_mode),
        _ => handle_unknown_message(message, output_mode),
    }
}

fn handle_trade_message(message: &StreamingMessage, output_mode: &OutputMode) -> Result<()> {
    let message_json = serde_json::to_value(message)?;
    match serde_json::from_value::<StreamingTrade>(message_json.clone()) {
        Ok(trade) => {
            let data = StreamingData {
                timestamp: Utc::now(),
                message_type: "t".to_string(),
                symbol: Some(trade.symbol.clone()),
                data: message_json,
            };
            output_mode.write_streaming_data(&data)?;
        }
        Err(e) => {
            eprintln!("❌ Failed to parse trade: {}", e);
        }
    }
    Ok(())
}

fn handle_quote_message(message: &StreamingMessage, output_mode: &OutputMode) -> Result<()> {
    let message_json = serde_json::to_value(message)?;
    match serde_json::from_value::<StreamingQuote>(message_json.clone()) {
        Ok(quote) => {
            let data = StreamingData {
                timestamp: Utc::now(),
                message_type: "q".to_string(),
                symbol: Some(quote.symbol.clone()),
                data: message_json,
            };
            output_mode.write_streaming_data(&data)?;
        }
        Err(e) => {
            eprintln!("❌ Failed to parse quote: {}", e);
        }
    }
    Ok(())
}

fn handle_bar_message(message: &StreamingMessage, output_mode: &OutputMode) -> Result<()> {
    let message_json = serde_json::to_value(message)?;
    match serde_json::from_value::<StreamingBar>(message_json.clone()) {
        Ok(bar) => {
            let data = StreamingData {
                timestamp: Utc::now(),
                message_type: "b".to_string(),
                symbol: Some(bar.symbol.clone()),
                data: message_json,
            };
            output_mode.write_streaming_data(&data)?;
        }
        Err(e) => {
            eprintln!("❌ Failed to parse bar: {}", e);
        }
    }
    Ok(())
}

fn handle_success_message(message: &StreamingMessage, output_mode: &OutputMode) -> Result<()> {
    if let Some(msg) = &message.message {
        let data = StreamingData {
            timestamp: Utc::now(),
            message_type: "success".to_string(),
            symbol: None,
            data: serde_json::Value::String(msg.clone()),
        };
        output_mode.write_streaming_data(&data)?;
    }
    Ok(())
}

fn handle_subscription_message(message: &StreamingMessage, output_mode: &OutputMode) -> Result<()> {
    if let Some(msg) = &message.message {
        let data = StreamingData {
            timestamp: Utc::now(),
            message_type: "subscription".to_string(),
            symbol: None,
            data: serde_json::Value::String(msg.clone()),
        };
        output_mode.write_streaming_data(&data)?;
    }
    Ok(())
}

fn handle_error_message(message: &StreamingMessage, output_mode: &OutputMode) -> Result<()> {
    if let Some(msg) = &message.message {
        let data = StreamingData {
            timestamp: Utc::now(),
            message_type: "error".to_string(),
            symbol: None,
            data: serde_json::Value::String(msg.clone()),
        };
        output_mode.write_streaming_data(&data)?;
    }
    Ok(())
}

fn handle_unknown_message(message: &StreamingMessage, output_mode: &OutputMode) -> Result<()> {
    let data = StreamingData {
        timestamp: Utc::now(),
        message_type: message.message_type.clone(),
        symbol: None,
        data: message.data.clone(),
    };
    output_mode.write_streaming_data(&data)?;
    Ok(())
}

pub fn get_symbols_from_env(env_var: &str, default: Vec<&str>) -> Vec<String> {
    match std::env::var(env_var) {
        Ok(symbols) => symbols.split(',').map(|s| s.trim().to_uppercase()).collect(),
        Err(_) => default.iter().map(|s| s.to_string()).collect(),
    }
}