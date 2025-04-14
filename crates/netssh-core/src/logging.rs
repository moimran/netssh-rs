use crate::error::NetsshError;
use chrono::Local;
use std::fs::create_dir_all;
use std::io;
use std::path::Path;
use tracing::metadata::LevelFilter;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan, time::FormatTime},
    prelude::*,
    EnvFilter, Layer,
};

struct CustomTime;

impl FormatTime for CustomTime {
    fn format_time(&self, w: &mut fmt::format::Writer<'_>) -> std::fmt::Result {
        write!(w, "{}", Local::now().format("%Y-%m-%d %H:%M:%S"))
    }
}

/// Initialize the tracing system for logging
///
/// # Arguments
///
/// * `level` - The log level to use ("error", "warn", "info", "debug", "trace")
/// * `log_to_file` - Whether to log to a file
/// * `log_file_path` - Path to the log file (only used if log_to_file is true)
/// * `log_format` - The format string for log messages (optional)
///
/// # Returns
///
/// Result indicating success or failure
pub fn init_logging(
    level: &str,
    log_to_file: bool,
    log_file_path: Option<&str>,
    _log_format: Option<&str>,
) -> Result<(), NetsshError> {
    // Determine log level from string
    let filter_level = match level.to_lowercase().as_str() {
        "error" => LevelFilter::ERROR,
        "warn" => LevelFilter::WARN,
        "info" => LevelFilter::INFO,
        "debug" => LevelFilter::DEBUG,
        "trace" => LevelFilter::TRACE,
        _ => LevelFilter::INFO,
    };

    // Create a filter based on RUST_LOG env var, or use our filter level
    let env_filter = match std::env::var("RUST_LOG") {
        Ok(env_val) => EnvFilter::new(env_val),
        Err(_) => EnvFilter::new(level.to_lowercase()),
    };

    // Configure logging based on whether file logging is enabled
    if log_to_file {
        // Determine log file path
        let log_path = match log_file_path {
            Some(path) => path.to_string(),
            None => {
                // Default to logs/debug.log
                let path = "logs/debug.log";

                // Create logs directory if it doesn't exist
                let dir = Path::new(path).parent().unwrap_or(Path::new(""));
                create_dir_all(dir).map_err(|e| NetsshError::IoError(e))?;

                path.to_string()
            }
        };

        // Create parent directory if it doesn't exist
        let dir = Path::new(&log_path).parent().unwrap_or(Path::new(""));
        create_dir_all(dir).map_err(|e| NetsshError::IoError(e))?;

        // Create a file for logging
        let log_file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path)
            .map_err(|e| NetsshError::IoError(e))?;

        // Create a custom file writer for logs
        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(log_file)
            .with_ansi(false) // Disable ANSI colors in file output
            .with_timer(CustomTime)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .with_span_events(FmtSpan::CLOSE)
            .with_filter(filter_level);

        // Create a standard stdout writer
        let stdout_layer = tracing_subscriber::fmt::layer()
            .with_writer(io::stdout)
            .with_ansi(true) // Enable ANSI colors for console output
            .with_timer(CustomTime)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .with_filter(filter_level);

        // Initialize with both console and file logging
        tracing_subscriber::registry()
            .with(env_filter)
            .with(file_layer)
            .with(stdout_layer)
            .init();
    } else {
        // Only initialize console logging if file logging is disabled
        tracing_subscriber::registry()
            .with(env_filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(io::stdout)
                    .with_ansi(true) // Enable ANSI colors for console output
                    .with_timer(CustomTime)
                    .with_thread_ids(true)
                    .with_thread_names(true)
                    .with_file(true)
                    .with_line_number(true)
                    .with_filter(filter_level),
            )
            .init();
    }

    tracing::info!("Logging initialized at {} level", level.to_uppercase());
    tracing::debug!("Debug logging is enabled");

    Ok(())
}

/// Creates a file writer for logs
#[allow(dead_code)]
fn create_log_writer(path: &str) -> Result<impl io::Write, NetsshError> {
    // Create parent directory if it doesn't exist
    let dir = Path::new(path).parent().unwrap_or(Path::new(""));
    create_dir_all(dir).map_err(|e| NetsshError::IoError(e))?;

    // Open the file
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(path)
        .map_err(|e| NetsshError::IoError(e))?;

    Ok(log_file)
}
