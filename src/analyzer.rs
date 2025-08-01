use algorithms_trading::{DataFormat, StreamingData};
use anyhow::Result;
use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::collections::HashMap;

#[derive(Parser, Debug)]
#[command(name = "data-analyzer")]
#[command(about = "Analyze streaming data files")]
#[command(version)]
struct Args {
    /// Input file to analyze
    #[arg(short, long)]
    input: PathBuf,
    
    /// Input format
    #[arg(short, long, value_enum, default_value_t = DataFormat::Json)]
    format: DataFormat,
}

#[derive(Debug, Default)]
struct DataStats {
    total_messages: u64,
    trade_count: u64,
    quote_count: u64,
    bar_count: u64,
    success_count: u64,
    subscription_count: u64,
    error_count: u64,
    symbol_counts: HashMap<String, u64>,
}

impl DataStats {
    fn add_message(&mut self, data: &StreamingData) {
        self.total_messages += 1;
        
        match data.message_type.as_str() {
            "t" => self.trade_count += 1,
            "q" => self.quote_count += 1,
            "b" => self.bar_count += 1,
            "success" => self.success_count += 1,
            "subscription" => self.subscription_count += 1,
            "error" => self.error_count += 1,
            _ => {}
        }
        
        if let Some(symbol) = &data.symbol {
            *self.symbol_counts.entry(symbol.clone()).or_insert(0) += 1;
        }
    }
    
    fn print_summary(&self) {
        println!("📊 Data Analysis Summary");
        println!("========================");
        println!("Total messages: {}", self.total_messages);
        println!("  Trades: {}", self.trade_count);
        println!("  Quotes: {}", self.quote_count);
        println!("  Bars: {}", self.bar_count);
        println!("  Success: {}", self.success_count);
        println!("  Subscription: {}", self.subscription_count);
        println!("  Errors: {}", self.error_count);
        
        if !self.symbol_counts.is_empty() {
            println!("\nSymbol breakdown:");
            let mut symbols: Vec<_> = self.symbol_counts.iter().collect();
            symbols.sort_by(|a, b| b.1.cmp(a.1));
            
            for (symbol, count) in symbols.iter().take(10) {
                println!("  {}: {}", symbol, count);
            }
            
            if symbols.len() > 10 {
                println!("  ... and {} more", symbols.len() - 10);
            }
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    println!("🔍 Analyzing data from: {}", args.input.display());
    println!("Format: {:?}\n", args.format);
    
    let mut stats = DataStats::default();
    
    match args.format {
        DataFormat::Json => {
            let file = File::open(&args.input)?;
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line?;
                if line.trim().is_empty() || !line.starts_with('{') {
                    continue; // Skip non-JSON lines (like status messages)
                }
                
                match serde_json::from_str::<StreamingData>(&line) {
                    Ok(data) => stats.add_message(&data),
                    Err(_) => continue, // Skip malformed JSON
                }
            }
        }
        DataFormat::Csv => {
            let file = File::open(&args.input)?;
            let mut csv_reader = csv::Reader::from_reader(file);
            for result in csv_reader.records() {
                let record = result?;
                if record.len() >= 4 {
                    if let Ok(timestamp) = record[0].parse::<chrono::DateTime<chrono::Utc>>() {
                        let data = StreamingData {
                            timestamp,
                            message_type: record[1].to_string(),
                            symbol: if record[2].is_empty() { None } else { Some(record[2].to_string()) },
                            data: serde_json::from_str(&record[3]).unwrap_or(serde_json::Value::Null),
                        };
                        stats.add_message(&data);
                    }
                }
            }
        }
        DataFormat::Plain => {
            println!("⚠️  Plain text format analysis is not supported yet.");
            println!("Please convert to JSON or CSV format first.");
            return Ok(());
        }
    }
    
    stats.print_summary();
    Ok(())
}