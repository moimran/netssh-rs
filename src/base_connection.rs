use crate::error::NetsshError;
use crate::session_log::SessionLog;
use crate::config::NetsshConfig;
use log::{debug, info};
use ssh2::Session;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{Duration, SystemTime};
use regex::Regex;

pub struct BaseConnection {
    pub session: Option<Session>,
    pub channel: Option<ssh2::Channel>,
    pub base_prompt: Option<String>,
    pub session_log: SessionLog,
    pub config: NetsshConfig,
}

impl BaseConnection {
    pub fn new() -> Result<Self, NetsshError> {
        debug!(target: "BaseConnection::new", "Creating new base connection");
        let config = NetsshConfig::default();
        let mut session_log = SessionLog::new();
        
        if config.enable_session_log {
            session_log.enable(&config.session_log_path)?;
        }
        
        Ok(BaseConnection {
            session: None,
            channel: None,
            base_prompt: None,
            session_log,
            config,
        })
    }

    pub fn with_config(config: NetsshConfig) -> Result<Self, NetsshError> {
        debug!(target: "BaseConnection::with_config", "Creating base connection with custom config");
        let mut session_log = SessionLog::new();
        
        if config.enable_session_log {
            session_log.enable(&config.session_log_path)?;
        }
        
        Ok(BaseConnection {
            session: None,
            channel: None,
            base_prompt: None,
            session_log,
            config,
        })
    }

    pub fn connect(
        &mut self,
        host: &str,
        username: &str,
        password: Option<&str>,
        port: Option<u16>,
        timeout: Option<Duration>,
    ) -> Result<(), NetsshError> {
        debug!(target: "BaseConnection::connect", "Connecting to {}@{}", username, host);
        
        let port = port.unwrap_or(self.config.default_port);
        let timeout = timeout.unwrap_or(self.config.connection_timeout);
        
        info!("Connecting to {}:{} with username {}", host, port, username);
        let addr = format!("{}:{}", host, port);
        
        debug!(target: "BaseConnection::connect", "Establishing TCP connection to {}", addr);
        let tcp = TcpStream::connect(&addr)
            .map_err(|e| {
                info!("Failed to establish TCP connection: {}", e);
                NetsshError::IoError(e)
            })?;
        
        debug!(target: "BaseConnection::connect", "TCP connection established");

        if let Some(timeout) = Some(timeout) {
            debug!(target: "BaseConnection::connect", "Setting TCP timeouts to {:?}", timeout);
            tcp.set_read_timeout(Some(self.config.read_timeout))
                .map_err(|e| NetsshError::IoError(e))?;
            tcp.set_write_timeout(Some(self.config.write_timeout))
                .map_err(|e| NetsshError::IoError(e))?;
        }

        debug!(target: "BaseConnection::connect", "Creating SSH session");
        let mut session = Session::new()
            .map_err(|e| {
                info!("Failed to create SSH session: {}", e);
                NetsshError::SshError(e)
            })?;
        session.set_tcp_stream(tcp);

        debug!(target: "BaseConnection::connect", "SSH session created, starting handshake");

        debug!(target: "BaseConnection::connect", "Starting SSH handshake");
        session.handshake()
            .map_err(|e| {
                info!("SSH handshake failed: {}", e);
                NetsshError::SshError(e)
            })?;

        debug!(target: "BaseConnection::connect", "SSH handshake completed successfully");

        debug!(target: "BaseConnection::connect", "Authenticating with username {}", username);
        if let Some(pass) = password {
            debug!(target: "BaseConnection::connect", "Attempting password authentication for user: {}", username);
            session.userauth_password(username, pass)
                .map_err(|e| {
                    info!("Password authentication failed: {}", e);
                    NetsshError::AuthenticationError(e.to_string())
                })?;
        } else {
            debug!(target: "BaseConnection::connect", "Attempting SSH agent authentication for user: {}", username);
            session.userauth_agent(username)
                .map_err(|e| {
                    info!("SSH agent authentication failed: {}", e);
                    NetsshError::AuthenticationError(e.to_string())
                })?;
        }

        debug!(target: "BaseConnection::connect", "Authentication successful");

        debug!(target: "BaseConnection::connect", "Opening SSH channel");
        let mut channel = session.channel_session()
            .map_err(|e| {
                info!("Failed to create channel session: {}", e);
                NetsshError::SshError(e)
            })?;

        debug!(target: "BaseConnection::connect", "SSH channel created successfully");

        debug!(target: "BaseConnection::connect", "Requesting PTY");
        channel.request_pty("xterm", None, None)
            .map_err(|e| {
                info!("Failed to request PTY: {}", e);
                NetsshError::SshError(e)
            })?;

        debug!(target: "BaseConnection::connect", "Starting shell");
        channel.shell()
            .map_err(|e| {
                info!("Failed to start shell: {}", e);
                NetsshError::SshError(e)
            })?;

        self.session = Some(session);
        self.channel = Some(channel);

        info!("Successfully connected to {}:{}", host, port);
        Ok(())
    }

    pub fn open_channel(&mut self) -> Result<(), NetsshError> {
        debug!(target: "BaseConnection::open_channel", "Opening SSH channel");
        let session = self.session.as_mut()
            .ok_or_else(|| {
                info!("Failed to open channel: no active session");
                NetsshError::ConnectionError("No active session".to_string())
            })?;

        let mut channel = session.channel_session()
            .map_err(|e| {
                info!("Failed to create channel session: {}", e);
                NetsshError::SshError(e)
            })?;

        debug!(target: "BaseConnection::open_channel", "Requesting PTY");
        channel.request_pty("xterm", None, None)
            .map_err(|e| {
                info!("Failed to request PTY: {}", e);
                NetsshError::SshError(e)
            })?;

        debug!(target: "BaseConnection::open_channel", "Starting shell");
        channel.shell()
            .map_err(|e| {
                info!("Failed to start shell: {}", e);
                NetsshError::SshError(e)
            })?;

        self.channel = Some(channel);
        debug!(target: "BaseConnection::open_channel", "Successfully opened channel and started shell");
        Ok(())
    }

    pub fn write_channel(&mut self, data: &str) -> Result<(), NetsshError> {
        debug!(target: "BaseConnection::write_channel", "Writing to channel: {:?}", data);
        let channel = self.channel.as_mut()
            .ok_or_else(|| NetsshError::ConnectionError("No active channel".to_string()))?;

        // Convert string to bytes and write to channel
        let bytes = data.as_bytes();
        channel.write_all(bytes)
            .map_err(|e| NetsshError::WriteError(format!("Failed to write to channel: {}", e)))?;

        // Flush the channel to ensure all data is sent
        channel.flush()
            .map_err(|e| NetsshError::WriteError(format!("Failed to flush channel: {}", e)))?;

        // Log the written data if session logging is enabled
        self.session_log.write_raw(bytes)?;

        debug!(target: "BaseConnection::write_channel", "Successfully wrote to channel");
        Ok(())
    }

    pub fn write_channel_raw(&mut self, data: &[u8]) -> Result<(), NetsshError> {
        debug!(target: "BaseConnection::write_channel_raw", "Writing raw bytes to channel: {:?}", data);
        let channel = self.channel.as_mut()
            .ok_or_else(|| NetsshError::ConnectionError("No active channel".to_string()))?;

        // Write raw bytes to channel
        channel.write_all(data)
            .map_err(|e| NetsshError::WriteError(format!("Failed to write to channel: {}", e)))?;

        // Flush the channel to ensure all data is sent
        channel.flush()
            .map_err(|e| NetsshError::WriteError(format!("Failed to flush channel: {}", e)))?;

        // Log the written data if session logging is enabled
        self.session_log.write_raw(data)?;

        debug!(target: "BaseConnection::write_channel_raw", "Successfully wrote raw bytes to channel");
        Ok(())
    }

    pub fn read_channel(&mut self) -> Result<String, NetsshError> {
        debug!(target: "BaseConnection::read_channel", "Reading from channel");
        
        let channel = self.channel.as_mut()
            .ok_or_else(|| NetsshError::ChannelError("No channel available".to_string()))?;

        let mut buffer = vec![0; self.config.read_buffer_size];
        let mut output = String::new();
        
        let start_time = SystemTime::now();
        
        while start_time.elapsed()? < self.config.read_timeout {
            match channel.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    let chunk = String::from_utf8_lossy(&buffer[..n]).to_string();
                    output.push_str(&chunk);
                    if self.session_log.is_active() {
                        self.session_log.write(&chunk)?;
                    }
                }
                Ok(_) => break,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(100));
                    continue;
                }
                Err(e) => return Err(NetsshError::IoError(e)),
            }
        }

        if output.is_empty() {
            debug!(target: "BaseConnection::read_channel", "No data read from channel");
        } else {
            debug!(target: "BaseConnection::read_channel", "Read {} bytes from channel", output.len());
        }

        Ok(output)
    }

    pub fn read_until_pattern(&mut self, pattern: &str) -> Result<String, NetsshError> {
        debug!(target: "BaseConnection::read_until_pattern", "Reading until pattern: {}", pattern);
        
        let regex = Regex::new(pattern)
            .map_err(|e| NetsshError::PatternError(e.to_string()))?;
        
        let mut output = String::new();
        let start_time = SystemTime::now();
        
        while start_time.elapsed()? < self.config.pattern_timeout {
            let chunk = self.read_channel()?;
            output.push_str(&chunk);
            
            if regex.is_match(&output) {
                debug!(target: "BaseConnection::read_until_pattern", "Pattern found");
                return Ok(output);
            }
            
            if chunk.is_empty() {
                std::thread::sleep(Duration::from_millis(100));
            }
        }
        
        Err(NetsshError::TimeoutError(format!(
            "Pattern '{}' not found within timeout period",
            pattern
        )))
    }

    pub fn send_command(&mut self, command: &str) -> Result<String, NetsshError> {
        debug!(target: "BaseConnection::send_command", "Sending command: {}", command);
        
        if self.config.auto_clear_buffer {
            // Clear any pending data in the buffer
            let _ = self.read_channel();
        }

        // Write the command
        self.write_channel(&format!("{}\n", command))?;

        // Try to read with retries
        let mut last_error = None;
        for retry in 0..self.config.retry_count {
            match self.read_channel() {
                Ok(output) => {
                    debug!(target: "BaseConnection::send_command", "Command output received");
                    return Ok(output);
                }
                Err(e) => {
                    debug!(target: "BaseConnection::send_command", 
                          "Retry {} failed: {}", retry + 1, e);
                    last_error = Some(e);
                    std::thread::sleep(self.config.retry_delay);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| 
            NetsshError::CommandError("Maximum retries exceeded".to_string())))
    }

    pub fn set_session_log(&mut self, filename: &str) -> Result<(), NetsshError> {
        self.session_log.enable(filename)
    }
}
