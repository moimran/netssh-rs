// Re-export shared configuration types for backward compatibility
pub use shared_config::{
    WorkspaceConfig, SchedulerConfig as SharedSchedulerConfig, DatabaseConfig, ServerConfig,
    WorkerConfig, BoardConfig, LoggingConfig, SchedulerServiceConfig,
    GlobalConfig, NetsshConfig, TextFsmConfig, FailureStrategy
};

use std::net::SocketAddr;

/// Legacy Config struct for backward compatibility
/// This wraps the shared configuration and provides the same interface
#[derive(Debug, Clone)]
pub struct Config {
    workspace_config: WorkspaceConfig,
}

// Re-export the scheduler service config as SchedulerConfig for backward compatibility
pub type SchedulerConfig = SchedulerServiceConfig;

impl Config {
    /// Load configuration from the unified workspace configuration
    pub fn from_env() -> Result<Self, shared_config::SharedConfigError> {
        let workspace_config = WorkspaceConfig::load()?;
        Ok(Self { workspace_config })
    }

    /// Get database configuration
    pub fn database(&self) -> &DatabaseConfig {
        &self.workspace_config.scheduler.database
    }

    /// Get server configuration
    pub fn server(&self) -> &ServerConfig {
        &self.workspace_config.scheduler.server
    }

    /// Get worker configuration
    pub fn worker(&self) -> &WorkerConfig {
        &self.workspace_config.scheduler.worker
    }

    /// Get board configuration
    pub fn board(&self) -> &BoardConfig {
        &self.workspace_config.scheduler.board
    }

    /// Get logging configuration
    pub fn logging(&self) -> &LoggingConfig {
        &self.workspace_config.scheduler.logging
    }

    /// Get scheduler service configuration
    pub fn scheduler(&self) -> &SchedulerServiceConfig {
        &self.workspace_config.scheduler.scheduler
    }

    /// Get global configuration
    pub fn global(&self) -> &GlobalConfig {
        &self.workspace_config.global
    }

    /// Get netssh configuration
    pub fn netssh(&self) -> &NetsshConfig {
        &self.workspace_config.netssh
    }

    /// Get the full workspace configuration
    pub fn workspace(&self) -> &WorkspaceConfig {
        &self.workspace_config
    }

    pub fn bind_address(&self) -> SocketAddr {
        format!("{}:{}", self.server().host, self.server().port)
            .parse()
            .expect("Invalid server address")
    }
}

impl Default for Config {
    fn default() -> Self {
        // Create a default workspace config and wrap it
        let workspace_config = WorkspaceConfig {
            global: GlobalConfig::default(),
            scheduler: SharedSchedulerConfig {
                enabled: true,
                database: DatabaseConfig {
                    url: "sqlite:scheduler.db".to_string(),
                    max_connections: 10,
                },
                server: ServerConfig {
                    host: "127.0.0.1".to_string(),
                    port: 8080,
                },
                worker: WorkerConfig {
                    concurrency: 4,
                    timeout_seconds: 300,
                    connection_reuse: true,
                    max_idle_time_seconds: 300,
                    max_connections_per_worker: 10,
                    failure_strategy: FailureStrategy::ContinueOnFailure,
                    failure_strategy_n: 3,
                },
                board: BoardConfig {
                    enabled: true,
                    ui_path: "/board".to_string(),
                    api_prefix: "/board/api".to_string(),
                    auth_enabled: false,
                },
                logging: LoggingConfig {
                    level: "info".to_string(),
                    file: None,
                    format: None,
                    rotation: None,
                },
                scheduler: SchedulerServiceConfig {
                    enabled: true,
                    poll_interval_seconds: 30,
                    timezone: Some("UTC".to_string()),
                    max_concurrent_jobs: 10,
                },
            },
            netssh: NetsshConfig {
                network: shared_config::NetworkSettings {
                    tcp_connect_timeout_secs: 60,
                    tcp_read_timeout_secs: 30,
                    tcp_write_timeout_secs: 30,
                    default_ssh_port: 22,
                    command_response_timeout_secs: 30,
                    pattern_match_timeout_secs: 20,
                    command_exec_delay_ms: 100,
                    retry_delay_ms: 1000,
                    max_retry_attempts: 3,
                    device_operation_timeout_secs: 120,
                },
                ssh: shared_config::SshSettings {
                    blocking_timeout_secs: 1,
                    auth_timeout_secs: 30,
                    keepalive_interval_secs: 60,
                    channel_open_timeout_secs: 20,
                },
                buffer: shared_config::BufferSettings {
                    read_buffer_size: 16384,
                    buffer_pool_size: 32,
                    buffer_reuse_threshold: 16384,
                    auto_clear_buffer: true,
                },
                concurrency: shared_config::ConcurrencySettings {
                    max_connections: 50,
                    permit_acquire_timeout_ms: 5000,
                    connection_idle_timeout_secs: 300,
                },
                logging: shared_config::NetsshLoggingConfig {
                    enable_session_log: false,
                    session_log_path: "logs".to_string(),
                    log_binary_data: false,
                },
                security: shared_config::SecurityConfig {
                    strict_host_key_checking: true,
                    known_hosts_file: "~/.ssh/known_hosts".to_string(),
                    max_auth_attempts: 3,
                },
            },
            textfsm: TextFsmConfig {
                template_cache_size: 1000,
                parsing_timeout_seconds: 10,
                template_directories: vec!["templates/".to_string()],
                enable_caching: true,
            },
        };

        Self { workspace_config }
    }
}
