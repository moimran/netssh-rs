use std::path::Path;
use tracing::Level;
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::config::LoggingConfig;

/// Initialize the logging system based on configuration
pub fn init_logging(config: &LoggingConfig) -> Result<(), Box<dyn std::error::Error>> {
    let _level = parse_log_level(&config.level)?;

    // Create environment filter with default level and SSH library logging
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| {
            // Configure logging for our application and SSH library
            // Enable debug level for netssh_core to see all SSH operations
            let ssh_level = if config.level == "trace" || config.level == "debug" {
                "debug"
            } else {
                "info"
            };

            let filter_str = format!(
                "{},netssh_core={},ssh2={},libssh2_sys={},sqlx=warn,apalis=info",
                &config.level,
                ssh_level, // netssh-core logs at info/debug level
                "debug",   // SSH2 library debug logs
                "warn",    // libssh2 system logs (can be noisy)
                           // sqlx=warn to reduce database polling noise
                           // apalis=info to show job processing but not polling
            );

            tracing::debug!("Logging filter configured: {}", filter_str);
            EnvFilter::try_new(&filter_str)
        })
        .unwrap_or_else(|_| {
            // Fallback with SSH logging enabled and reduced database noise
            EnvFilter::new("info,netssh_core=debug,ssh2=debug,sqlx=warn,apalis=info")
        });

    match (&config.file, config.format.as_deref().unwrap_or("text")) {
        (Some(file_path), "json") => {
            // JSON file logging with rotation
            let path = Path::new(file_path);
            let directory = path.parent().unwrap_or_else(|| Path::new("."));
            let filename = path
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("scheduler.log"));

            let file_appender = rolling::daily(directory, filename);
            let (file_writer, _guard) = non_blocking(file_appender);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt::layer().json().with_writer(file_writer))
                .with(fmt::layer().json())
                .init();
        }
        (Some(file_path), _) => {
            // Text file logging with rotation
            let path = Path::new(file_path);
            let directory = path.parent().unwrap_or_else(|| Path::new("."));
            let filename = path
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("scheduler.log"));

            let file_appender = rolling::daily(directory, filename);
            let (file_writer, _guard) = non_blocking(file_appender);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt::layer().with_writer(file_writer))
                .with(fmt::layer())
                .init();
        }
        (None, "json") => {
            // JSON console logging only
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt::layer().json())
                .init();
        }
        (None, _) => {
            // Text console logging only
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt::layer())
                .init();
        }
    }

    tracing::info!(
        level = %config.level,
        file = ?config.file,
        format = %config.format.as_deref().unwrap_or("text"),
        ssh_logging = "enabled",
        "Logging system initialized"
    );

    // Log SSH library information
    log_ssh_library_info();

    Ok(())
}

/// Log information about SSH library integration
fn log_ssh_library_info() {
    tracing::info!(
        netssh_core = "enabled",
        ssh2 = "enabled",
        libssh2 = "enabled",
        sqlx_level = "warn",
        apalis_level = "info",
        "SSH library logging configured - netssh-core logs will appear with 'netssh_core' target"
    );

    tracing::debug!("SSH library stack: netssh-core -> ssh2 -> libssh2-sys");

    tracing::info!(
        "Database polling noise reduced (sqlx=warn), job processing visible (apalis=info)"
    );
}

/// Parse log level from string
fn parse_log_level(level: &str) -> Result<Level, Box<dyn std::error::Error>> {
    match level.to_lowercase().as_str() {
        "trace" => Ok(Level::TRACE),
        "debug" => Ok(Level::DEBUG),
        "info" => Ok(Level::INFO),
        "warn" | "warning" => Ok(Level::WARN),
        "error" => Ok(Level::ERROR),
        _ => Err(format!("Invalid log level: {}", level).into()),
    }
}

/// Macro for structured logging with job context
#[macro_export]
macro_rules! log_job_event {
    ($level:ident, job_id = $job_id:expr, $($key:ident = $value:expr),* $(,)?) => {
        tracing::$level!(
            job_id = %$job_id,
            $($key = $value),*
        );
    };
    ($level:ident, job_id = $job_id:expr, $message:expr, $($key:ident = $value:expr),* $(,)?) => {
        tracing::$level!(
            job_id = %$job_id,
            $($key = $value),*,
            $message
        );
    };
}

/// Macro for structured logging with SSH context
#[macro_export]
macro_rules! log_ssh_event {
    ($level:ident, host = $host:expr, $($key:ident = $value:expr),* $(,)?) => {
        tracing::$level!(
            ssh_host = %$host,
            $($key = $value),*
        );
    };
    ($level:ident, host = $host:expr, $message:expr, $($key:ident = $value:expr),* $(,)?) => {
        tracing::$level!(
            ssh_host = %$host,
            $($key = $value),*,
            $message
        );
    };
}

/// Macro for structured logging with API context
#[macro_export]
macro_rules! log_api_event {
    ($level:ident, method = $method:expr, path = $path:expr, $($key:ident = $value:expr),* $(,)?) => {
        tracing::$level!(
            http_method = %$method,
            http_path = %$path,
            $($key = $value),*
        );
    };
    ($level:ident, method = $method:expr, path = $path:expr, $message:expr, $($key:ident = $value:expr),* $(,)?) => {
        tracing::$level!(
            http_method = %$method,
            http_path = %$path,
            $($key = $value),*,
            $message
        );
    };
}

/// Macro for structured logging with database context
#[macro_export]
macro_rules! log_db_event {
    ($level:ident, operation = $operation:expr, $($key:ident = $value:expr),* $(,)?) => {
        tracing::$level!(
            db_operation = %$operation,
            $($key = $value),*
        );
    };
    ($level:ident, operation = $operation:expr, $message:expr, $($key:ident = $value:expr),* $(,)?) => {
        tracing::$level!(
            db_operation = %$operation,
            $($key = $value),*,
            $message
        );
    };
}
