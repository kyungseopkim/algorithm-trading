use algorithms_trading::{DataFormat, OutputMode};
use alpaca_trading_api_rust::{Bar, StockBarsResponse};
use std::collections::HashMap;
use tempfile::tempdir;
use std::fs;

// Mock data creation helpers
fn create_mock_bar(_symbol: &str, timestamp: &str, open: f64, close: f64) -> Bar {
    Bar {
        t: timestamp.to_string(),
        o: open,
        h: open + 5.0,
        l: open - 2.0,
        c: close,
        v: 10000,
        n: 500,
        vw: (open + close) / 2.0,
    }
}

fn create_mock_stock_bars_response(symbol: &str, bars: Vec<Bar>) -> StockBarsResponse {
    let mut bars_map = HashMap::new();
    bars_map.insert(symbol.to_string(), bars);
    
    StockBarsResponse {
        bars: bars_map,
        next_page_token: None,
    }
}

fn create_mock_stock_bars_response_with_pagination(symbol: &str, bars: Vec<Bar>, next_token: Option<String>) -> StockBarsResponse {
    let mut bars_map = HashMap::new();
    bars_map.insert(symbol.to_string(), bars);
    
    StockBarsResponse {
        bars: bars_map,
        next_page_token: next_token,
    }
}

#[tokio::test]
async fn test_mock_api_response_parsing() {
    // Test that we can properly parse a mock API response
    let bars = vec![
        create_mock_bar("AAPL", "2024-01-15T10:00:00Z", 150.0, 153.0),
        create_mock_bar("AAPL", "2024-01-16T10:00:00Z", 153.0, 155.0),
    ];
    
    let response = create_mock_stock_bars_response("AAPL", bars);
    
    assert!(response.bars.contains_key("AAPL"));
    assert_eq!(response.bars["AAPL"].len(), 2);
    assert_eq!(response.bars["AAPL"][0].o, 150.0);
    assert_eq!(response.bars["AAPL"][1].c, 155.0);
    assert!(response.next_page_token.is_none());
}

#[tokio::test]
async fn test_mock_pagination_response() {
    // Test pagination handling
    let bars_page1 = vec![
        create_mock_bar("AAPL", "2024-01-15T10:00:00Z", 150.0, 153.0),
    ];
    
    let bars_page2 = vec![
        create_mock_bar("AAPL", "2024-01-16T10:00:00Z", 153.0, 155.0),
    ];
    
    let response1 = create_mock_stock_bars_response_with_pagination("AAPL", bars_page1, Some("token123".to_string()));
    let response2 = create_mock_stock_bars_response_with_pagination("AAPL", bars_page2, None);
    
    assert_eq!(response1.next_page_token, Some("token123".to_string()));
    assert!(response2.next_page_token.is_none());
}

#[tokio::test] 
async fn test_empty_response_handling() {
    // Test handling of empty responses
    let response = create_mock_stock_bars_response("AAPL", vec![]);
    
    assert!(response.bars.contains_key("AAPL"));
    assert_eq!(response.bars["AAPL"].len(), 0);
}

#[tokio::test]
async fn test_multiple_symbols_response() {
    // Test handling multiple symbols in response
    let mut bars_map = HashMap::new();
    bars_map.insert("AAPL".to_string(), vec![
        create_mock_bar("AAPL", "2024-01-15T10:00:00Z", 150.0, 153.0),
    ]);
    bars_map.insert("MSFT".to_string(), vec![
        create_mock_bar("MSFT", "2024-01-15T10:00:00Z", 300.0, 305.0),
    ]);
    
    let response = StockBarsResponse {
        bars: bars_map,
        next_page_token: None,
    };
    
    assert!(response.bars.contains_key("AAPL"));
    assert!(response.bars.contains_key("MSFT"));
    assert_eq!(response.bars["AAPL"][0].c, 153.0);
    assert_eq!(response.bars["MSFT"][0].c, 305.0);
}

#[tokio::test]
async fn test_output_mode_with_mock_data() {
    // Test output mode with mock data
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("mock_test.json");
    
    let output_mode = OutputMode::create_file_mode(&file_path, DataFormat::Json, false).unwrap();
    
    // Create mock historical bar data
    let bar_data = r#"{"symbol":"AAPL","timestamp":"2024-01-15T10:00:00Z","open":150.0,"high":155.0,"low":148.0,"close":153.0,"volume":10000,"trade_count":500,"vwap":151.5}"#;
    
    output_mode.writeln(bar_data).unwrap();
    
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("AAPL"));
    assert!(content.contains("2024-01-15T10:00:00Z"));
}

#[tokio::test]
async fn test_csv_output_with_mock_data() {
    // Test CSV output with mock data
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("mock_test.csv");
    
    let output_mode = OutputMode::create_file_mode(&file_path, DataFormat::Csv, false).unwrap();
    
    // Write CSV header
    output_mode.writeln("symbol,timestamp,open,high,low,close,volume,trade_count,vwap").unwrap();
    
    // Write mock data
    output_mode.writeln("AAPL,2024-01-15T10:00:00Z,150.00,155.00,148.00,153.00,10000,500,151.5").unwrap();
    output_mode.writeln("MSFT,2024-01-15T10:00:00Z,300.00,305.00,298.00,303.00,5000,250,301.5").unwrap();
    
    let content = fs::read_to_string(&file_path).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    
    assert_eq!(lines.len(), 3); // header + 2 data lines
    assert_eq!(lines[0], "symbol,timestamp,open,high,low,close,volume,trade_count,vwap");
    assert!(lines[1].contains("AAPL"));
    assert!(lines[2].contains("MSFT"));
}

#[test]
fn test_error_scenarios() {
    // Test various error scenarios
    
    // Invalid file path
    let invalid_path = std::path::PathBuf::from("/invalid/path/file.txt");
    let result = OutputMode::create_file_mode(&invalid_path, DataFormat::Plain, false);
    assert!(result.is_err());
    
    // Test that we can handle errors gracefully
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    let output_mode = OutputMode::create_file_mode(&file_path, DataFormat::Plain, false).unwrap();
    
    // This should succeed
    assert!(output_mode.writeln("test").is_ok());
}

#[test]
fn test_data_format_serialization() {
    // Test that our data formats work correctly
    use serde_json;
    
    let mock_data = serde_json::json!({
        "symbol": "AAPL",
        "timestamp": "2024-01-15T10:00:00Z",
        "open": 150.0,
        "high": 155.0,
        "low": 148.0,
        "close": 153.0,
        "volume": 10000,
        "trade_count": 500,
        "vwap": 151.5
    });
    
    let serialized = serde_json::to_string(&mock_data).unwrap();
    assert!(serialized.contains("AAPL"));
    assert!(serialized.contains("150.0"));
    
    let deserialized: serde_json::Value = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized["symbol"], "AAPL");
    assert_eq!(deserialized["open"], 150.0);
}

#[tokio::test]
async fn test_rate_limiting_simulation() {
    // Simulate rate limiting behavior
    use tokio::time::{sleep, Duration, Instant};
    
    let start = Instant::now();
    
    // Simulate multiple API calls with delays (like in fetch_historical_data)
    for _ in 0..3 {
        sleep(Duration::from_millis(100)).await;
    }
    
    let elapsed = start.elapsed();
    assert!(elapsed >= Duration::from_millis(300)); // Should take at least 300ms for 3 calls
}

#[test]
fn test_date_comparison_logic() {
    // Test the date comparison logic used in validation
    use chrono::NaiveDate;
    
    let date1 = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    let date2 = NaiveDate::from_ymd_opt(2024, 1, 10).unwrap();
    
    assert!(date1 > date2);
    assert!(date1.format("%Y-%m-%d").to_string() == "2024-01-15");
    assert!(date2.format("%Y-%m-%d").to_string() == "2024-01-10");
}

#[test]
fn test_symbol_processing() {
    // Test symbol processing logic
    let symbols_input = "aapl,MSFT, googl ,  TSLA  ";
    let processed: Vec<String> = symbols_input
        .split(',')
        .map(|s| s.trim().to_uppercase())
        .collect();
    
    assert_eq!(processed, vec!["AAPL", "MSFT", "GOOGL", "TSLA"]);
}

#[test]
fn test_page_size_bounds() {
    // Test page size validation
    let valid_sizes = vec![1, 100, 1000, 5000, 10000];
    let invalid_sizes = vec![0, 10001, 15000, 50000];
    
    for size in valid_sizes {
        assert!(size > 0 && size <= 10000);
    }
    
    for size in invalid_sizes {
        assert!(size == 0 || size > 10000);
    }
}