use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SharedConfigError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Environment error: {0}")]
    Environment(String),
}

pub type Result<T> = std::result::Result<T, SharedConfigError>;

/// Global configuration that applies to all crates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub log_level: String,
    pub environment: String,
    pub default_timeout_seconds: u64,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            environment: "development".to_string(),
            default_timeout_seconds: 30,
        }
    }
}

/// Scheduler-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    pub enabled: bool,
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub worker: WorkerConfig,
    pub board: BoardConfig,
    pub logging: LoggingConfig,
    pub scheduler: SchedulerServiceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerConfig {
    pub concurrency: usize,
    pub timeout_seconds: u64,
    pub connection_reuse: bool,
    pub max_idle_time_seconds: u64,
    pub max_connections_per_worker: usize,
    pub failure_strategy: FailureStrategy,
    pub failure_strategy_n: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureStrategy {
    ContinueOnFailure,
    AbortOnFirstFailure,
    AbortAfterNFailures(usize),
}

impl std::fmt::Display for FailureStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FailureStrategy::ContinueOnFailure => write!(f, "continue"),
            FailureStrategy::AbortOnFirstFailure => write!(f, "abort_first"),
            FailureStrategy::AbortAfterNFailures(n) => write!(f, "abort_after_{}", n),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardConfig {
    pub enabled: bool,
    pub ui_path: String,
    pub api_prefix: String,
    pub auth_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file: Option<String>,
    pub format: Option<String>,
    pub rotation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerServiceConfig {
    pub enabled: bool,
    pub poll_interval_seconds: u64,
    pub timezone: Option<String>,
    pub max_concurrent_jobs: u32,
}

/// netssh-core configuration - comprehensive settings matching the original settings.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetsshConfig {
    pub network: NetworkSettings,
    pub ssh: SshSettings,
    pub buffer: BufferSettings,
    pub concurrency: ConcurrencySettings,
    pub logging: NetsshLoggingConfig,
    pub security: SecurityConfig,
}

/// Network-related timeout settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSettings {
    /// TCP connection timeout in seconds (default: 60)
    pub tcp_connect_timeout_secs: u64,
    /// TCP read timeout in seconds (default: 30)
    pub tcp_read_timeout_secs: u64,
    /// TCP write timeout in seconds (default: 30)
    pub tcp_write_timeout_secs: u64,
    /// Default SSH port (default: 22)
    pub default_ssh_port: u16,
    /// Command response timeout in seconds (default: 30)
    pub command_response_timeout_secs: u64,
    /// Pattern match timeout in seconds (default: 20)
    pub pattern_match_timeout_secs: u64,
    /// Command execution delay in milliseconds (default: 100)
    pub command_exec_delay_ms: u64,
    /// Retry delay in milliseconds (default: 1000)
    pub retry_delay_ms: u64,
    /// Maximum retry attempts (default: 3)
    pub max_retry_attempts: u32,
    /// Device operation timeout in seconds (default: 120)
    pub device_operation_timeout_secs: u64,
}

/// SSH-related settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshSettings {
    /// Timeout for blocking libssh2 function calls in seconds (default: 1)
    /// Set to 0 for no timeout
    pub blocking_timeout_secs: u64,
    /// SSH authentication timeout in seconds (default: 30)
    pub auth_timeout_secs: u64,
    /// SSH keepalive interval in seconds (default: 60)
    pub keepalive_interval_secs: u64,
    /// SSH channel open timeout in seconds (default: 20)
    pub channel_open_timeout_secs: u64,
}

/// Buffer-related settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferSettings {
    /// Read buffer size in bytes (default: 65536)
    pub read_buffer_size: usize,
    /// Buffer pool size (default: 32)
    pub buffer_pool_size: usize,
    /// Buffer reuse threshold in bytes (default: 16384)
    pub buffer_reuse_threshold: usize,
    /// Whether to automatically clear buffer before commands (default: true)
    pub auto_clear_buffer: bool,
}

/// Concurrency-related settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencySettings {
    /// Maximum number of concurrent connections (default: 100)
    pub max_connections: usize,
    /// Permit acquire timeout in milliseconds (default: 5000)
    pub permit_acquire_timeout_ms: u64,
    /// Connection idle timeout in seconds (default: 300)
    pub connection_idle_timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetsshLoggingConfig {
    /// Whether to enable session logging (default: false)
    pub enable_session_log: bool,
    /// Path to the session log directory (default: "logs")
    pub session_log_path: String,
    /// Whether to log binary data (default: false)
    pub log_binary_data: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub strict_host_key_checking: bool,
    pub known_hosts_file: String,
    pub max_auth_attempts: u32,
}

/// TextFSM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextFsmConfig {
    pub template_cache_size: usize,
    pub parsing_timeout_seconds: u64,
    pub template_directories: Vec<String>,
    pub enable_caching: bool,
}

/// Main configuration structure that contains all crate configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub global: GlobalConfig,
    pub scheduler: SchedulerConfig,
    pub netssh: NetsshConfig,
    pub textfsm: TextFsmConfig,
}

impl WorkspaceConfig {
    /// Load configuration from file and environment variables
    pub fn load() -> Result<Self> {
        let mut config_builder = Config::builder();

        // Add default configuration file from workspace root
        let workspace_root = find_workspace_root()?;
        let config_file = workspace_root.join("config.toml");
        
        if config_file.exists() {
            config_builder = config_builder.add_source(File::from(config_file));
        }

        // Add environment variables with NETSSH prefix
        config_builder = config_builder.add_source(
            Environment::with_prefix("NETSSH")
                .separator("_")
                .try_parsing(true)
        );

        let config = config_builder.build()?;

        // Build the configuration with defaults
        let workspace_config = WorkspaceConfig {
            global: config.get("global").unwrap_or_default(),
            scheduler: SchedulerConfig {
                enabled: config.get("scheduler.enabled").unwrap_or(true),
                database: DatabaseConfig {
                    url: config.get("scheduler.database.url").unwrap_or_else(|_| "sqlite:scheduler.db".to_string()),
                    max_connections: config.get("scheduler.database.max_connections").unwrap_or(10),
                },
                server: ServerConfig {
                    host: config.get("scheduler.server.host").unwrap_or_else(|_| "127.0.0.1".to_string()),
                    port: config.get("scheduler.server.port").unwrap_or(8080),
                },
                worker: WorkerConfig {
                    concurrency: config.get("scheduler.worker.concurrency").unwrap_or(4),
                    timeout_seconds: config.get("scheduler.worker.timeout_seconds").unwrap_or(300),
                    connection_reuse: config.get("scheduler.worker.connection_reuse").unwrap_or(true),
                    max_idle_time_seconds: config.get("scheduler.worker.max_idle_time_seconds").unwrap_or(300),
                    max_connections_per_worker: config.get("scheduler.worker.max_connections_per_worker").unwrap_or(10),
                    failure_strategy: match config.get("scheduler.worker.failure_strategy").unwrap_or_else(|_| "continue".to_string()).as_str() {
                        "abort_first" => FailureStrategy::AbortOnFirstFailure,
                        "abort_after_n" => {
                            let n = config.get("scheduler.worker.failure_strategy_n").unwrap_or(3);
                            FailureStrategy::AbortAfterNFailures(n)
                        },
                        _ => FailureStrategy::ContinueOnFailure,
                    },
                    failure_strategy_n: config.get("scheduler.worker.failure_strategy_n").unwrap_or(3),
                },
                board: BoardConfig {
                    enabled: config.get("scheduler.board.enabled").unwrap_or(true),
                    ui_path: config.get("scheduler.board.ui_path").unwrap_or_else(|_| "/board".to_string()),
                    api_prefix: config.get("scheduler.board.api_prefix").unwrap_or_else(|_| "/board/api".to_string()),
                    auth_enabled: config.get("scheduler.board.auth_enabled").unwrap_or(false),
                },
                logging: LoggingConfig {
                    level: config.get("scheduler.logging.level").unwrap_or_else(|_| "info".to_string()),
                    file: config.get("scheduler.logging.file").ok(),
                    format: config.get("scheduler.logging.format").ok(),
                    rotation: config.get("scheduler.logging.rotation").ok(),
                },
                scheduler: SchedulerServiceConfig {
                    enabled: config.get("scheduler.scheduler.enabled").unwrap_or(true),
                    poll_interval_seconds: config.get("scheduler.scheduler.poll_interval_seconds").unwrap_or(30),
                    timezone: config.get("scheduler.scheduler.timezone").ok(),
                    max_concurrent_jobs: config.get("scheduler.scheduler.max_concurrent_jobs").unwrap_or(10),
                },
            },
            netssh: NetsshConfig {
                network: NetworkSettings {
                    tcp_connect_timeout_secs: config.get("netssh.network.tcp_connect_timeout_secs").unwrap_or(60),
                    tcp_read_timeout_secs: config.get("netssh.network.tcp_read_timeout_secs").unwrap_or(30),
                    tcp_write_timeout_secs: config.get("netssh.network.tcp_write_timeout_secs").unwrap_or(30),
                    default_ssh_port: config.get("netssh.network.default_ssh_port").unwrap_or(22),
                    command_response_timeout_secs: config.get("netssh.network.command_response_timeout_secs").unwrap_or(30),
                    pattern_match_timeout_secs: config.get("netssh.network.pattern_match_timeout_secs").unwrap_or(20),
                    command_exec_delay_ms: config.get("netssh.network.command_exec_delay_ms").unwrap_or(100),
                    retry_delay_ms: config.get("netssh.network.retry_delay_ms").unwrap_or(1000),
                    max_retry_attempts: config.get("netssh.network.max_retry_attempts").unwrap_or(3),
                    device_operation_timeout_secs: config.get("netssh.network.device_operation_timeout_secs").unwrap_or(120),
                },
                ssh: SshSettings {
                    blocking_timeout_secs: config.get("netssh.ssh.blocking_timeout_secs").unwrap_or(1),
                    auth_timeout_secs: config.get("netssh.ssh.auth_timeout_secs").unwrap_or(30),
                    keepalive_interval_secs: config.get("netssh.ssh.keepalive_interval_secs").unwrap_or(60),
                    channel_open_timeout_secs: config.get("netssh.ssh.channel_open_timeout_secs").unwrap_or(20),
                },
                buffer: BufferSettings {
                    read_buffer_size: config.get("netssh.buffer.read_buffer_size").unwrap_or(65536),
                    buffer_pool_size: config.get("netssh.buffer.buffer_pool_size").unwrap_or(32),
                    buffer_reuse_threshold: config.get("netssh.buffer.buffer_reuse_threshold").unwrap_or(16384),
                    auto_clear_buffer: config.get("netssh.buffer.auto_clear_buffer").unwrap_or(true),
                },
                concurrency: ConcurrencySettings {
                    max_connections: config.get("netssh.concurrency.max_connections").unwrap_or(100),
                    permit_acquire_timeout_ms: config.get("netssh.concurrency.permit_acquire_timeout_ms").unwrap_or(5000),
                    connection_idle_timeout_secs: config.get("netssh.concurrency.connection_idle_timeout_secs").unwrap_or(300),
                },
                logging: NetsshLoggingConfig {
                    enable_session_log: config.get("netssh.logging.enable_session_log").unwrap_or(false),
                    session_log_path: config.get("netssh.logging.session_log_path").unwrap_or_else(|_| "logs".to_string()),
                    log_binary_data: config.get("netssh.logging.log_binary_data").unwrap_or(false),
                },
                security: SecurityConfig {
                    strict_host_key_checking: config.get("netssh.security.strict_host_key_checking").unwrap_or(true),
                    known_hosts_file: config.get("netssh.security.known_hosts_file").unwrap_or_else(|_| "~/.ssh/known_hosts".to_string()),
                    max_auth_attempts: config.get("netssh.security.max_auth_attempts").unwrap_or(3),
                },
            },
            textfsm: TextFsmConfig {
                template_cache_size: config.get("textfsm.template_cache_size").unwrap_or(1000),
                parsing_timeout_seconds: config.get("textfsm.parsing_timeout_seconds").unwrap_or(10),
                template_directories: config.get("textfsm.template_directories").unwrap_or_else(|_| vec!["templates/".to_string()]),
                enable_caching: config.get("textfsm.enable_caching").unwrap_or(true),
            },
        };

        Ok(workspace_config)
    }

    /// Get scheduler-specific configuration
    pub fn scheduler(&self) -> &SchedulerConfig {
        &self.scheduler
    }

    /// Get netssh-core configuration
    pub fn netssh(&self) -> &NetsshConfig {
        &self.netssh
    }

    /// Get TextFSM configuration
    pub fn textfsm(&self) -> &TextFsmConfig {
        &self.textfsm
    }

    /// Get global configuration
    pub fn global(&self) -> &GlobalConfig {
        &self.global
    }
}

/// Find the workspace root directory by looking for Cargo.toml with [workspace]
fn find_workspace_root() -> Result<PathBuf> {
    let mut current_dir = std::env::current_dir()?;
    
    loop {
        let cargo_toml = current_dir.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = std::fs::read_to_string(&cargo_toml)?;
            if content.contains("[workspace]") {
                return Ok(current_dir);
            }
        }
        
        if let Some(parent) = current_dir.parent() {
            current_dir = parent.to_path_buf();
        } else {
            break;
        }
    }
    
    Err(SharedConfigError::Environment(
        "Could not find workspace root directory".to_string()
    ))
}
