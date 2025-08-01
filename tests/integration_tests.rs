use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_cli_help_command() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "historical-data", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Retrieve historical bar data for symbols"));
    assert!(stdout.contains("--symbols <SYMBOLS>"));
    assert!(stdout.contains("--start <START>"));
    assert!(stdout.contains("--end <END>"));
    assert!(stdout.contains("--timeframe <TIMEFRAME>"));
    assert!(stdout.contains("--format <FORMAT>"));
}

#[test]
fn test_cli_version_command() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "historical-data", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("historical-data"));
}

#[test]
fn test_cli_missing_required_args() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "historical-data", "--", "--symbols", "AAPL"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("required") || stderr.contains("missing"));
}

#[test]
fn test_cli_invalid_date_range() {
    let output = Command::new("cargo")
        .args(&[
            "run", "--bin", "historical-data", "--",
            "--symbols", "AAPL",
            "--start", "2024-01-15",
            "--end", "2024-01-10"
        ])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Start date must be before end date"));
}

#[test]
fn test_cli_invalid_timeframe() {
    let output = Command::new("cargo")
        .args(&[
            "run", "--bin", "historical-data", "--",
            "--symbols", "AAPL",
            "--start", "2024-01-01",
            "--end", "2024-01-02",
            "--timeframe", "invalid"
        ])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Invalid timeframe"));
}

#[test]
fn test_cli_invalid_date_format() {
    let output = Command::new("cargo")
        .args(&[
            "run", "--bin", "historical-data", "--",
            "--symbols", "AAPL",
            "--start", "01-01-2024",
            "--end", "01-02-2024"
        ])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Error") || stderr.contains("Invalid"));
}

#[test]
fn test_cli_page_size_validation() {
    let output = Command::new("cargo")
        .args(&[
            "run", "--bin", "historical-data", "--",
            "--symbols", "AAPL",
            "--start", "2024-01-01",
            "--end", "2024-01-02",
            "--page-size", "15000"
        ])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Page size cannot exceed 10000"));
}

#[test]
fn test_cli_output_file_creation() {
    let temp_dir = tempdir().unwrap();
    let output_file = temp_dir.path().join("test_output.txt");
    
    // This test would require valid API credentials to run fully
    // So we'll just test that the file path is accepted by the CLI parser
    let output = Command::new("cargo")
        .args(&[
            "run", "--bin", "historical-data", "--",
            "--symbols", "AAPL",
            "--start", "2024-01-01",
            "--end", "2024-01-02",
            "--output", output_file.to_str().unwrap(),
            "--format", "json"
        ])
        .output()
        .expect("Failed to execute command");

    // The command should fail due to missing API credentials, but not due to invalid arguments
    let stderr = String::from_utf8(output.stderr).unwrap();
    // Should not contain argument parsing errors
    assert!(!stderr.contains("required") && !stderr.contains("invalid"));
}

#[test]
fn test_cli_multiple_symbols() {
    let output = Command::new("cargo")
        .args(&[
            "run", "--bin", "historical-data", "--",
            "--symbols", "AAPL,MSFT,GOOGL",
            "--start", "2024-01-01",
            "--end", "2024-01-02",
            "--format", "csv"
        ])
        .output()
        .expect("Failed to execute command");

    // The command should fail due to missing API credentials, but not due to invalid arguments
    let stderr = String::from_utf8(output.stderr).unwrap();
    // Should not contain argument parsing errors
    assert!(!stderr.contains("required") && !stderr.contains("invalid"));
}

#[test]
fn test_cli_all_format_options() {
    for format in &["plain", "json", "csv"] {
        let output = Command::new("cargo")
            .args(&[
                "run", "--bin", "historical-data", "--",
                "--symbols", "AAPL",
                "--start", "2024-01-01",
                "--end", "2024-01-02",
                "--format", format
            ])
            .output()
            .expect("Failed to execute command");

        // The command should fail due to missing API credentials, but not due to invalid format
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(!stderr.contains("invalid format") && !stderr.contains("Unknown variant"));
    }
}

#[test]
fn test_cli_all_timeframe_options() {
    for timeframe in &["1Min", "5Min", "15Min", "30Min", "1Hour", "1Day", "1Week", "1Month"] {
        let output = Command::new("cargo")
            .args(&[
                "run", "--bin", "historical-data", "--",
                "--symbols", "AAPL",
                "--start", "2024-01-01",
                "--end", "2024-01-02",
                "--timeframe", timeframe
            ])
            .output()
            .expect("Failed to execute command");

        // The command should fail due to missing API credentials, but not due to invalid timeframe
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(!stderr.contains("Invalid timeframe"));
    }
}

#[test] 
fn test_cli_append_flag() {
    let temp_dir = tempdir().unwrap();
    let output_file = temp_dir.path().join("test_append.txt");
    
    let output = Command::new("cargo")
        .args(&[
            "run", "--bin", "historical-data", "--",
            "--symbols", "AAPL",
            "--start", "2024-01-01", 
            "--end", "2024-01-02",
            "--output", output_file.to_str().unwrap(),
            "--append"
        ])
        .output()
        .expect("Failed to execute command");

    // The command should fail due to missing API credentials, but not due to invalid arguments
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(!stderr.contains("required") && !stderr.contains("invalid"));
}

#[test]
fn test_cli_invalid_feed() {
    let output = Command::new("cargo")
        .args(&[
            "run", "--bin", "historical-data", "--",
            "--symbols", "AAPL",
            "--start", "2024-01-01",
            "--end", "2024-01-02",
            "--feed", "invalid"
        ])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Invalid feed"));
}

#[test]
fn test_cli_all_feed_options() {
    for feed in &["sip", "iex", "boats", "otc"] {
        let output = Command::new("cargo")
            .args(&[
                "run", "--bin", "historical-data", "--",
                "--symbols", "AAPL",
                "--start", "2024-01-01",
                "--end", "2024-01-02",
                "--feed", feed
            ])
            .output()
            .expect("Failed to execute command");

        // The command should fail due to missing API credentials, but not due to invalid feed
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(!stderr.contains("Invalid feed"));
    }
}