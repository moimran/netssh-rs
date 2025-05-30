use shared_config::{TextFsmConfig, WorkspaceConfig, SharedConfigError};
use std::path::PathBuf;

/// TextFSM configuration wrapper that provides access to shared configuration
/// while maintaining backward compatibility with existing TextFSM functionality
#[derive(Debug, Clone)]
pub struct Config {
    workspace_config: WorkspaceConfig,
}

impl Config {
    /// Load configuration from environment variables and config files
    pub fn from_env() -> Result<Self, SharedConfigError> {
        let workspace_config = WorkspaceConfig::load()?;
        Ok(Self { workspace_config })
    }

    /// Get the TextFSM configuration section
    pub fn textfsm(&self) -> &TextFsmConfig {
        &self.workspace_config.textfsm
    }

    /// Get the full workspace configuration
    pub fn workspace(&self) -> &WorkspaceConfig {
        &self.workspace_config
    }

    /// Get template cache size
    pub fn template_cache_size(&self) -> usize {
        self.textfsm().template_cache_size
    }

    /// Get parsing timeout in seconds
    pub fn parsing_timeout_seconds(&self) -> u64 {
        self.textfsm().parsing_timeout_seconds
    }

    /// Get template directories
    pub fn template_directories(&self) -> &[String] {
        &self.textfsm().template_directories
    }

    /// Check if caching is enabled
    pub fn enable_caching(&self) -> bool {
        self.textfsm().enable_caching
    }

    /// Get template directories as PathBuf vector
    pub fn template_paths(&self) -> Vec<PathBuf> {
        self.template_directories()
            .iter()
            .map(|dir| PathBuf::from(dir))
            .collect()
    }

    /// Find template file in configured directories
    pub fn find_template(&self, template_name: &str) -> Option<PathBuf> {
        for dir in self.template_directories() {
            let template_path = PathBuf::from(dir).join(template_name);
            if template_path.exists() {
                return Some(template_path);
            }
        }
        None
    }

    /// Get default template directory (first in the list)
    pub fn default_template_directory(&self) -> Option<&str> {
        self.template_directories().first().map(|s| s.as_str())
    }
}

impl Default for Config {
    fn default() -> Self {
        // Create a default workspace config with sensible TextFSM defaults
        Self {
            workspace_config: WorkspaceConfig::load().unwrap_or_else(|_| {
                // If loading fails, create a minimal config with defaults
                WorkspaceConfig {
                    global: shared_config::GlobalConfig::default(),
                    scheduler: shared_config::SchedulerConfig {
                        enabled: false,
                        database: shared_config::DatabaseConfig {
                            url: "sqlite::memory:".to_string(),
                            max_connections: 1,
                        },
                        server: shared_config::ServerConfig {
                            host: "127.0.0.1".to_string(),
                            port: 8080,
                        },
                        worker: shared_config::WorkerConfig {
                            concurrency: 1,
                            timeout_seconds: 300,
                            connection_reuse: false,
                            max_idle_time_seconds: 300,
                            max_connections_per_worker: 1,
                            failure_strategy: shared_config::FailureStrategy::ContinueOnFailure,
                            failure_strategy_n: 3,
                        },
                        board: shared_config::BoardConfig {
                            enabled: false,
                            ui_path: "/board".to_string(),
                            api_prefix: "/board/api".to_string(),
                            auth_enabled: false,
                        },
                        logging: shared_config::LoggingConfig {
                            level: "info".to_string(),
                            file: None,
                            format: None,
                            rotation: None,
                        },
                        scheduler: shared_config::SchedulerServiceConfig {
                            enabled: false,
                            poll_interval_seconds: 30,
                            timezone: None,
                            max_concurrent_jobs: 1,
                        },
                    },
                    netssh: shared_config::NetsshConfig {
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
                            max_connections: 1,
                            permit_acquire_timeout_ms: 5000,
                            connection_idle_timeout_secs: 300,
                        },
                        logging: shared_config::NetsshLoggingConfig {
                            enable_session_log: false,
                            session_log_path: "logs".to_string(),
                            log_binary_data: false,
                        },
                        security: shared_config::SecurityConfig {
                            strict_host_key_checking: false,
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
                }
            }),
        }
    }
}

/// Configuration builder for TextFSM settings
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    template_cache_size: Option<usize>,
    parsing_timeout_seconds: Option<u64>,
    template_directories: Option<Vec<String>>,
    enable_caching: Option<bool>,
}

impl ConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set template cache size
    pub fn template_cache_size(mut self, size: usize) -> Self {
        self.template_cache_size = Some(size);
        self
    }

    /// Set parsing timeout in seconds
    pub fn parsing_timeout_seconds(mut self, timeout: u64) -> Self {
        self.parsing_timeout_seconds = Some(timeout);
        self
    }

    /// Set template directories
    pub fn template_directories(mut self, dirs: Vec<String>) -> Self {
        self.template_directories = Some(dirs);
        self
    }

    /// Add a template directory
    pub fn add_template_directory(mut self, dir: String) -> Self {
        if let Some(ref mut dirs) = self.template_directories {
            dirs.push(dir);
        } else {
            self.template_directories = Some(vec![dir]);
        }
        self
    }

    /// Enable or disable caching
    pub fn enable_caching(mut self, enable: bool) -> Self {
        self.enable_caching = Some(enable);
        self
    }

    /// Build the configuration
    pub fn build(self) -> Config {
        let mut workspace_config = Config::default().workspace_config;
        
        // Apply builder settings to the TextFSM config
        if let Some(cache_size) = self.template_cache_size {
            workspace_config.textfsm.template_cache_size = cache_size;
        }
        if let Some(timeout) = self.parsing_timeout_seconds {
            workspace_config.textfsm.parsing_timeout_seconds = timeout;
        }
        if let Some(dirs) = self.template_directories {
            workspace_config.textfsm.template_directories = dirs;
        }
        if let Some(enable) = self.enable_caching {
            workspace_config.textfsm.enable_caching = enable;
        }

        Config { workspace_config }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.template_cache_size() > 0);
        assert!(config.parsing_timeout_seconds() > 0);
        assert!(!config.template_directories().is_empty());
        assert!(config.enable_caching());
    }

    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new()
            .template_cache_size(500)
            .parsing_timeout_seconds(30)
            .add_template_directory("custom_templates/".to_string())
            .enable_caching(false)
            .build();

        assert_eq!(config.template_cache_size(), 500);
        assert_eq!(config.parsing_timeout_seconds(), 30);
        assert!(config.template_directories().contains(&"custom_templates/".to_string()));
        assert!(!config.enable_caching());
    }

    #[test]
    fn test_template_paths() {
        let config = ConfigBuilder::new()
            .add_template_directory("templates/".to_string())
            .add_template_directory("/usr/share/textfsm/".to_string())
            .build();

        let paths = config.template_paths();
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0], PathBuf::from("templates/"));
        assert_eq!(paths[1], PathBuf::from("/usr/share/textfsm/"));
    }

    #[test]
    fn test_default_template_directory() {
        let config = ConfigBuilder::new()
            .add_template_directory("first/".to_string())
            .add_template_directory("second/".to_string())
            .build();

        assert_eq!(config.default_template_directory(), Some("first/"));
    }
}
