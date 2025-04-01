use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::RwLock;
use std::time::Duration;
use tracing::{debug, error};

/// Global Settings for netssh-rs
/// This file provides a central place to configure all timeout values and other settings
/// that might need to be adjusted for different environments.
///
/// Settings can be loaded from a TOML file, JSON file, or environment variables.
/// Default values are provided for all settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Network-related timeouts
    pub network: NetworkSettings,

    /// SSH-related settings
    pub ssh: SshSettings,

    /// Buffer settings
    pub buffer: BufferSettings,

    /// Concurrency settings
    pub concurrency: ConcurrencySettings,

    /// Logging settings
    pub logging: LoggingSettings,
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

    /// Default port for SSH connections (default: 22)
    pub default_ssh_port: u16,

    /// Command response timeout in seconds (default: 30)
    /// How long to wait for a response after sending a command
    pub command_response_timeout_secs: u64,

    /// Pattern matching timeout in seconds (default: 20)
    /// How long to wait for a pattern match when reading output
    pub pattern_match_timeout_secs: u64,

    /// Command execution delay in milliseconds (default: 100)
    /// Short delay between sending a command and starting to read the response
    pub command_exec_delay_ms: u64,

    /// Delay between retry attempts in milliseconds (default: 1000)
    pub retry_delay_ms: u64,

    /// Maximum number of retry attempts (default: 3)
    pub max_retry_attempts: u32,

    /// Timeout for device-specific operations (default: 120)
    /// Used for operations that might take longer on certain device types
    pub device_operation_timeout_secs: u64,
}

/// SSH-related settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshSettings {
    /// Timeout for blocking libssh2 function calls in seconds (default: 30)
    /// Set to 0 for no timeout
    pub blocking_timeout_secs: u64,

    /// SSH authentication timeout in seconds (default: 30)
    pub auth_timeout_secs: u64,

    /// SSH keepalive interval in seconds (default: 60)
    /// How often to send keepalive packets
    pub keepalive_interval_secs: u64,

    /// SSH channel open timeout in seconds (default: 20)
    pub channel_open_timeout_secs: u64,
}

/// Buffer-related settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferSettings {
    /// Default read buffer size in bytes (default: 65536)
    pub read_buffer_size: usize,

    /// Maximum buffer pool size (default: 32)
    /// Number of buffers to keep in the pool
    pub buffer_pool_size: usize,

    /// Buffer reuse threshold in bytes (default: 16384)
    /// Buffers smaller than this will be reused, larger ones will be allocated
    pub buffer_reuse_threshold: usize,

    /// Whether to automatically clear the buffer before sending commands (default: true)
    pub auto_clear_buffer: bool,
}

/// Concurrency-related settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencySettings {
    /// Maximum number of concurrent connections (default: 100)
    pub max_connections: usize,

    /// Timeout for acquiring a connection permit in milliseconds (default: 5000)
    pub permit_acquire_timeout_ms: u64,

    /// Connection pool idle timeout in seconds (default: 300)
    /// How long a connection can remain idle before being closed
    pub connection_idle_timeout_secs: u64,
}

/// Logging-related settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingSettings {
    /// Whether to enable session logging (default: false)
    pub enable_session_log: bool,

    /// Path to the session log directory (default: "logs")
    pub session_log_path: String,

    /// Whether to log binary data (default: false)
    pub log_binary_data: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            network: NetworkSettings::default(),
            ssh: SshSettings::default(),
            buffer: BufferSettings::default(),
            concurrency: ConcurrencySettings::default(),
            logging: LoggingSettings::default(),
        }
    }
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
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
        }
    }
}

impl Default for SshSettings {
    fn default() -> Self {
        Self {
            blocking_timeout_secs: 30,
            auth_timeout_secs: 30,
            keepalive_interval_secs: 60,
            channel_open_timeout_secs: 20,
        }
    }
}

impl Default for BufferSettings {
    fn default() -> Self {
        Self {
            read_buffer_size: 65536, // 64KB
            buffer_pool_size: 32,
            buffer_reuse_threshold: 16384, // 16KB
            auto_clear_buffer: true,
        }
    }
}

impl Default for ConcurrencySettings {
    fn default() -> Self {
        Self {
            max_connections: 100,
            permit_acquire_timeout_ms: 5000,
            connection_idle_timeout_secs: 300,
        }
    }
}

impl Default for LoggingSettings {
    fn default() -> Self {
        Self {
            enable_session_log: false,
            session_log_path: String::from("logs"),
            log_binary_data: false,
        }
    }
}

// Global instance of Settings with RwLock for thread-safe access
lazy_static! {
    pub static ref SETTINGS: RwLock<Settings> = RwLock::new(Settings::default());
}

impl Settings {
    /// Load settings from a file
    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let path = Path::new(path);
        if !path.exists() {
            return Err(format!("Settings file not found: {}", path.display()));
        }

        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) => return Err(format!("Failed to read settings file: {}", e)),
        };

        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            Self::load_from_json(&content)
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("toml") {
            Self::load_from_toml(&content)
        } else {
            Err(format!("Unsupported file format: {:?}", path.extension()))
        }
    }

    /// Load settings from JSON string
    pub fn load_from_json(json: &str) -> Result<Self, String> {
        match serde_json::from_str::<Settings>(json) {
            Ok(settings) => Ok(settings),
            Err(e) => Err(format!("Failed to parse JSON settings: {}", e)),
        }
    }

    /// Load settings from TOML string
    pub fn load_from_toml(_toml: &str) -> Result<Self, String> {
        // TODO: Implement TOML parsing once the format is finalized
        Err("TOML loading not yet implemented".to_string())
    }

    /// Initialize global settings
    pub fn init(path: Option<&str>) -> Result<(), String> {
        let settings = if let Some(path) = path {
            Self::load_from_file(path)?
        } else {
            Settings::default()
        };

        // Update the global settings
        let mut global_settings = SETTINGS.write().map_err(|e| e.to_string())?;
        *global_settings = settings;

        debug!("Settings initialized successfully");
        Ok(())
    }

    /// Get a copy of the current settings
    pub fn get() -> Result<Settings, String> {
        let settings = SETTINGS.read().map_err(|e| e.to_string())?;
        Ok(settings.clone())
    }

    /// Update specific settings
    pub fn update<F>(updater: F) -> Result<(), String>
    where
        F: FnOnce(&mut Settings),
    {
        let mut settings = SETTINGS.write().map_err(|e| e.to_string())?;
        updater(&mut settings);
        debug!("Settings updated successfully");
        Ok(())
    }
}

/// Helper function to get duration from settings
pub fn get_network_timeout(timeout_type: NetworkTimeoutType) -> Duration {
    let settings = match SETTINGS.read() {
        Ok(settings) => settings,
        Err(_) => {
            error!("Failed to access global settings, using defaults");
            return match timeout_type {
                NetworkTimeoutType::TcpConnect => Duration::from_secs(60),
                NetworkTimeoutType::TcpRead => Duration::from_secs(30),
                NetworkTimeoutType::TcpWrite => Duration::from_secs(30),
                NetworkTimeoutType::CommandResponse => Duration::from_secs(30),
                NetworkTimeoutType::PatternMatch => Duration::from_secs(20),
                NetworkTimeoutType::DeviceOperation => Duration::from_secs(120),
            };
        }
    };

    match timeout_type {
        NetworkTimeoutType::TcpConnect => {
            Duration::from_secs(settings.network.tcp_connect_timeout_secs)
        }
        NetworkTimeoutType::TcpRead => Duration::from_secs(settings.network.tcp_read_timeout_secs),
        NetworkTimeoutType::TcpWrite => {
            Duration::from_secs(settings.network.tcp_write_timeout_secs)
        }
        NetworkTimeoutType::CommandResponse => {
            Duration::from_secs(settings.network.command_response_timeout_secs)
        }
        NetworkTimeoutType::PatternMatch => {
            Duration::from_secs(settings.network.pattern_match_timeout_secs)
        }
        NetworkTimeoutType::DeviceOperation => {
            Duration::from_secs(settings.network.device_operation_timeout_secs)
        }
    }
}

/// Types of network timeouts
pub enum NetworkTimeoutType {
    TcpConnect,
    TcpRead,
    TcpWrite,
    CommandResponse,
    PatternMatch,
    DeviceOperation,
}

/// Types of SSH timeouts
pub enum SshTimeoutType {
    Blocking,
    Auth,
    ChannelOpen,
    KeepaliveInterval,
}

/// Helper function to get SSH timeouts
pub fn get_ssh_timeout(timeout_type: SshTimeoutType) -> Duration {
    let settings = match SETTINGS.read() {
        Ok(settings) => settings,
        Err(_) => {
            error!("Failed to access global settings, using defaults");
            return match timeout_type {
                SshTimeoutType::Blocking => Duration::from_secs(1),
                SshTimeoutType::Auth => Duration::from_secs(30),
                SshTimeoutType::ChannelOpen => Duration::from_secs(20),
                SshTimeoutType::KeepaliveInterval => Duration::from_secs(60),
            };
        }
    };

    match timeout_type {
        SshTimeoutType::Blocking => Duration::from_secs(settings.ssh.blocking_timeout_secs),
        SshTimeoutType::Auth => Duration::from_secs(settings.ssh.auth_timeout_secs),
        SshTimeoutType::ChannelOpen => Duration::from_secs(settings.ssh.channel_open_timeout_secs),
        SshTimeoutType::KeepaliveInterval => {
            Duration::from_secs(settings.ssh.keepalive_interval_secs)
        }
    }
}

/// Helper function to get concurrency settings
pub fn get_concurrency_setting(setting_type: ConcurrencySettingType) -> u64 {
    let settings = match SETTINGS.read() {
        Ok(settings) => settings,
        Err(_) => {
            error!("Failed to access global settings, using defaults");
            return match setting_type {
                ConcurrencySettingType::MaxConnections => 100,
                ConcurrencySettingType::PermitAcquireTimeoutMs => 5000,
                ConcurrencySettingType::ConnectionIdleTimeoutSecs => 300,
            };
        }
    };

    match setting_type {
        ConcurrencySettingType::MaxConnections => settings.concurrency.max_connections as u64,
        ConcurrencySettingType::PermitAcquireTimeoutMs => {
            settings.concurrency.permit_acquire_timeout_ms
        }
        ConcurrencySettingType::ConnectionIdleTimeoutSecs => {
            settings.concurrency.connection_idle_timeout_secs
        }
    }
}

/// Types of concurrency settings
pub enum ConcurrencySettingType {
    MaxConnections,
    PermitAcquireTimeoutMs,
    ConnectionIdleTimeoutSecs,
}

/// Helper function to get buffer settings
pub fn get_buffer_setting(setting_type: BufferSettingType) -> usize {
    let settings = match SETTINGS.read() {
        Ok(settings) => settings,
        Err(_) => {
            error!("Failed to access global settings, using defaults");
            return match setting_type {
                BufferSettingType::ReadBufferSize => 65536,
                BufferSettingType::BufferPoolSize => 32,
                BufferSettingType::BufferReuseThreshold => 16384,
            };
        }
    };

    match setting_type {
        BufferSettingType::ReadBufferSize => settings.buffer.read_buffer_size,
        BufferSettingType::BufferPoolSize => settings.buffer.buffer_pool_size,
        BufferSettingType::BufferReuseThreshold => settings.buffer.buffer_reuse_threshold,
    }
}

/// Types of buffer settings
pub enum BufferSettingType {
    ReadBufferSize,
    BufferPoolSize,
    BufferReuseThreshold,
}
