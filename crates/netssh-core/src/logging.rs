use crate::error::NetsshError;
use chrono::Local;
use std::fs::create_dir_all;
use std::io;
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
/// * `debug_enabled` - Whether debug level logging should be enabled
/// * `_session_logging_enabled` - Session logging is now handled by BaseConnection
///
/// # Returns
///
/// Result indicating success or failure
pub fn init_logging(
    debug_enabled: bool,
    _session_logging_enabled: bool, // This is now handled by BaseConnection
) -> Result<(), NetsshError> {
    // Create logs directory if it doesn't exist
    create_dir_all("logs").map_err(|e| NetsshError::IoError(e))?;

    // Determine log level based on debug flag
    let filter_level = if debug_enabled {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };

    // Create a filter based on RUST_LOG env var, or use our default
    let env_filter = match std::env::var("RUST_LOG") {
        Ok(env_val) => EnvFilter::new(env_val),
        Err(_) => {
            if debug_enabled {
                EnvFilter::new("debug")
            } else {
                EnvFilter::new("info")
            }
        }
    };

    // Create a file for logging
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("logs/debug.log")
        .map_err(|e| NetsshError::IoError(e))?;

    // Create a custom file writer for the debug logs
    let file_appender = tracing_subscriber::fmt::layer()
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

    // Register both writers with the subscriber
    tracing_subscriber::registry()
        .with(env_filter)
        .with(file_appender)
        .with(stdout_layer)
        .init();

    tracing::info!(
        "Logging initialized at {} level",
        if debug_enabled { "DEBUG" } else { "INFO" }
    );
    tracing::debug!("Debug logging is enabled");

    Ok(())
}

/// Creates a debug file writer
fn create_debug_writer() -> Result<impl io::Write, NetsshError> {
    let debug_path = "logs/debug.log";
    let debug_file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(debug_path)
        .map_err(|e| NetsshError::IoError(e))?;

    Ok(debug_file)
}
