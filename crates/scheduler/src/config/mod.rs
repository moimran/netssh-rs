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
                default_ssh_timeout: 30,
                default_command_timeout: 60,
                default_port: 22,
                buffer_size: 16384,
                max_retries: 3,
                connection_pool_size: 50,
                logging: shared_config::NetsshLoggingConfig {
                    session_logging: false,
                    command_logging: true,
                    performance_logging: true,
                    debug_mode: false,
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
