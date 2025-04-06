use crate::settings::{get_ssh_timeout, SshTimeoutType};
use std::time::Duration;

/// Configuration settings for Netssh-RS connections
#[derive(Debug, Clone)]
pub struct NetsshConfig {
    /// Hostname or IP address of target device
    pub host: String,

    /// Username to authenticate with
    pub username: String,

    /// Password for authentication (optional)
    pub password: Option<String>,

    /// Enable secret for privilege escalation (optional)
    pub secret: Option<String>,

    /// Default SSH port if not specified (default: 22)
    pub default_port: u16,

    /// Global connection timeout in seconds (default: 60)
    /// This affects both TCP connection and SSH handshake timeouts
    pub connection_timeout: Duration,

    /// Read timeout for channel operations in seconds (default: 10)
    /// Used when reading from SSH channels
    pub read_timeout: Duration,

    /// Write timeout for channel operations in seconds (default: 10)
    /// Used when writing to SSH channels
    pub write_timeout: Duration,

    /// Size of the read buffer in bytes (default: 65536)
    /// Larger values may improve performance but use more memory
    pub read_buffer_size: usize,

    /// Maximum time to wait for a pattern match in seconds (default: 20)
    /// Used in read_until_pattern operations
    pub pattern_timeout: Duration,

    /// Whether to automatically clear the buffer before sending commands (default: true)
    pub auto_clear_buffer: bool,

    /// Number of retries for failed operations (default: 3)
    pub retry_count: u32,

    /// Delay between retries in milliseconds (default: 1000)
    pub retry_delay: Duration,

    /// Whether to enable session logging (default: false)
    pub enable_session_log: bool,

    /// Path to the session log file (default: "logs/session.log")
    pub session_log_path: String,

    /// Timeout for blocking SSH operations in seconds (default: 30)
    /// Used for all blocking libssh2 function calls (read/write/auth/etc)
    /// Set to 0 for no timeout
    pub blocking_timeout: Duration,
}

impl Default for NetsshConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            username: String::new(),
            password: None,
            secret: None,
            default_port: 22,
            connection_timeout: Duration::from_secs(60),
            read_timeout: Duration::from_secs(10),
            write_timeout: Duration::from_secs(10),
            read_buffer_size: 65536, // 64KB
            pattern_timeout: Duration::from_secs(20),
            auto_clear_buffer: true,
            retry_count: 3,
            retry_delay: Duration::from_millis(1000),
            enable_session_log: true,
            session_log_path: String::from("logs/session.log"),
            blocking_timeout: get_ssh_timeout(SshTimeoutType::Blocking),
        }
    }
}

impl NetsshConfig {
    /// Creates a new NetsshConfig with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a builder for NetsshConfig to allow fluent configuration
    pub fn builder() -> NetsshConfigBuilder {
        NetsshConfigBuilder::default()
    }
}

/// Builder for NetsshConfig to allow fluent configuration
#[derive(Default)]
pub struct NetsshConfigBuilder {
    config: NetsshConfig,
}

impl NetsshConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn default_port(mut self, port: u16) -> Self {
        self.config.default_port = port;
        self
    }

    pub fn connection_timeout(mut self, timeout: Duration) -> Self {
        self.config.connection_timeout = timeout;
        self
    }

    pub fn read_timeout(mut self, timeout: Duration) -> Self {
        self.config.read_timeout = timeout;
        self
    }

    pub fn write_timeout(mut self, timeout: Duration) -> Self {
        self.config.write_timeout = timeout;
        self
    }

    pub fn read_buffer_size(mut self, size: usize) -> Self {
        self.config.read_buffer_size = size;
        self
    }

    pub fn pattern_timeout(mut self, timeout: Duration) -> Self {
        self.config.pattern_timeout = timeout;
        self
    }

    pub fn auto_clear_buffer(mut self, enable: bool) -> Self {
        self.config.auto_clear_buffer = enable;
        self
    }

    pub fn retry_count(mut self, count: u32) -> Self {
        self.config.retry_count = count;
        self
    }

    pub fn retry_delay(mut self, delay: Duration) -> Self {
        self.config.retry_delay = delay;
        self
    }

    pub fn enable_session_log(mut self, enable: bool) -> Self {
        self.config.enable_session_log = enable;
        self
    }

    pub fn session_log_path(mut self, path: String) -> Self {
        self.config.session_log_path = path;
        self
    }

    pub fn host(mut self, host: String) -> Self {
        self.config.host = host;
        self
    }

    pub fn username(mut self, username: String) -> Self {
        self.config.username = username;
        self
    }

    pub fn password(mut self, password: String) -> Self {
        self.config.password = Some(password);
        self
    }

    pub fn secret(mut self, secret: String) -> Self {
        self.config.secret = Some(secret);
        self
    }

    pub fn build(self) -> NetsshConfig {
        self.config
    }
}
