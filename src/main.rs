use algorithms_trading::{DataFormat, OutputMode, StreamingConfig, run_streaming_client};
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tokio::time::sleep;
use std::time::Duration;
use dotenv::dotenv;

#[derive(Parser, Debug)]
#[command(name = "algorithms-trading")]
#[command(about = "Alpaca Trading API streaming client")]
#[command(version)]
struct Args {
    /// Output to file instead of console
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    /// Append to existing file instead of overwriting
    #[arg(short, long)]
    append: bool,
    
    /// Data format for output
    #[arg(short, long, value_enum, default_value_t = DataFormat::Plain)]
    format: DataFormat,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    
    let args = Args::parse();
    
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    // Create output mode based on arguments
    let output_mode = if let Some(output_path) = args.output {
        println!("📁 Output will be written to: {} in {:?} format", output_path.display(), args.format);
        OutputMode::create_file_mode(&output_path, args.format, args.append)?
    } else {
        OutputMode::create_console_mode(args.format)
    };
    
    output_mode.writeln("🚀 Starting Advanced Alpaca Streaming Example with Reconnection...")?;
    
    let config = StreamingConfig::new(output_mode);
    let mut retry_count = 0;
    
    loop {
        match run_streaming_client(&config).await {
            Ok(_) => {
                config.output_mode.writeln("✅ Streaming session completed successfully")?;
                break;
            }
            Err(e) => {
                retry_count += 1;
                eprintln!("❌ Streaming error (attempt {}/{}): {}", retry_count, config.max_retries, e);
                
                if retry_count >= config.max_retries {
                    eprintln!("🔴 Max retries reached. Exiting...");
                    return Err(e);
                }
                
                let backoff_duration = Duration::from_secs(2_u64.pow(retry_count.min(6)));
                config.output_mode.writeln(&format!("⏳ Retrying in {} seconds...", backoff_duration.as_secs()))?;
                sleep(backoff_duration).await;
            }
        }
    }
    
    Ok(())
}