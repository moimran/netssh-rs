use crate::channel::SSHChannel;
use crate::config::NetsshConfig;
use crate::error::NetsshError;
use crate::patterns::{ANSI_ESCAPE_PATTERN, CRLF_PATTERN};
use crate::session_log::SessionLog;
use log::{debug, info, warn};
use regex::Regex;
use ssh2::Session;
use std::net::TcpStream;
use std::thread;
use std::time::{Duration, SystemTime};
use rand;

pub struct BaseConnection {
    pub session: Option<Session>,
    pub channel: SSHChannel,
    pub base_prompt: Option<String>,
    pub session_log: SessionLog,
    pub config: NetsshConfig,
    pub disable_lf_normalization: bool,
    pub ansi_escape_codes: bool,
    pub read_timeout_override: Option<Duration>,
    _read_buffer: String,
}

// Constants for sleep durations
const DEFAULT_COMMAND_WAIT_MS: u64 = 500;
const DEFAULT_LOOP_DELAY_MS: u64 = 10;

impl BaseConnection {
    // Helper method to sleep for a configurable duration
    fn sleep_for_command(&self, duration_ms: Option<u64>) {
        let duration = Duration::from_millis(duration_ms.unwrap_or(DEFAULT_COMMAND_WAIT_MS));
        debug!(target: "BaseConnection::sleep_for_command", "Sleeping for {:?}", duration);
        thread::sleep(duration);
    }
    pub fn new() -> Result<Self, NetsshError> {
        debug!(target: "BaseConnection::new", "Creating new base connection");
        let config = NetsshConfig::default();
        let mut session_log = SessionLog::new();

        if config.enable_session_log {
            session_log.enable(&config.session_log_path)?;
        }

        Ok(BaseConnection {
            session: None,
            channel: SSHChannel::new(None),
            base_prompt: None,
            session_log,
            config,
            disable_lf_normalization: false,
            ansi_escape_codes: false,
            read_timeout_override: None,
            _read_buffer: String::new(),
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
            channel: SSHChannel::new(None),
            base_prompt: None,
            session_log,
            config,
            disable_lf_normalization: false,
            ansi_escape_codes: false,
            read_timeout_override: None,
            _read_buffer: String::new(),
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
        let tcp = match TcpStream::connect(&addr) {
            Ok(stream) => {
                debug!(target: "BaseConnection::connect", "TCP connection established");
                stream
            },
            Err(e) => {
                info!("Failed to establish TCP connection: {}", e);
                return Err(NetsshError::connection_failed(addr, e));
            }
        };

        debug!(target: "BaseConnection::connect", "Setting TCP timeouts to {:?}", timeout);
        if let Err(e) = tcp.set_read_timeout(Some(self.config.read_timeout)) {
            return Err(NetsshError::IoError(e));
        }
        
        if let Err(e) = tcp.set_write_timeout(Some(self.config.write_timeout)) {
            return Err(NetsshError::IoError(e));
        }

        debug!(target: "BaseConnection::connect", "Creating SSH session");
        let mut session = match Session::new() {
            Ok(session) => session,
            Err(e) => {
                info!("Failed to create SSH session: {}", e);
                return Err(NetsshError::SshError(e));
            }
        };
        session.set_tcp_stream(tcp);

        debug!(target: "BaseConnection::connect", "Starting SSH handshake");
        if let Err(e) = session.handshake() {
            info!("SSH handshake failed: {}", e);
            return Err(NetsshError::ssh_handshake_failed(e));
        }

        debug!(target: "BaseConnection::connect", "SSH handshake completed successfully");
        debug!(target: "BaseConnection::connect", "Authenticating with username {}", username);
        
        if let Some(pass) = password {
            debug!(target: "BaseConnection::connect", "Attempting password authentication for user: {}", username);
            if let Err(e) = session.userauth_password(username, pass) {
                info!("Password authentication failed: {}", e);
                return Err(NetsshError::authentication_failed(username, e));
            }
        } else {
            debug!(target: "BaseConnection::connect", "Attempting SSH agent authentication for user: {}", username);
            if let Err(e) = session.userauth_agent(username) {
                info!("SSH agent authentication failed: {}", e);
                return Err(NetsshError::authentication_failed(username, e));
            }
        }

        debug!(target: "BaseConnection::connect", "Authentication successful");
        debug!(target: "BaseConnection::connect", "Opening SSH channel");
        
        let mut channel = match session.channel_session() {
            Ok(channel) => {
                debug!(target: "BaseConnection::connect", "SSH channel created successfully");
                channel
            },
            Err(e) => {
                info!("Failed to create channel session: {}", e);
                return Err(NetsshError::channel_failed("Failed to create channel session", Some(e)));
            }
        };

        debug!(target: "BaseConnection::connect", "Requesting PTY");
        channel.request_pty("xterm", None, None).map_err(|e| {
            info!("Failed to request PTY: {}", e);
            NetsshError::SshError(e)
        })?;

        debug!(target: "BaseConnection::connect", "Starting shell");
        channel.shell().map_err(|e| {
            info!("Failed to start shell: {}", e);
            NetsshError::SshError(e)
        })?;

        session.set_blocking(true);

        // After successful connection, enable keepalive mechanism
        debug!(target: "BaseConnection::connect", "Enabling SSH keep-alive");
        session.set_keepalive(true, 60); // Send keepalive every 60 seconds
        
        // Store the session
        self.session = Some(session);
        self.channel = SSHChannel::new(Some(channel));
        
        debug!(target: "BaseConnection::connect", "Connection established successfully");
        Ok(())
    }

    pub fn open_channel(&mut self) -> Result<(), NetsshError> {
        debug!(target: "BaseConnection::open_channel", "Opening SSH channel");
        let session = self.session.as_mut().ok_or_else(|| {
            info!("Failed to open channel: no active session");
            NetsshError::ConnectionError("No active session".to_string())
        })?;

        let mut channel = session.channel_session().map_err(|e| {
            info!("Failed to create channel session: {}", e);
            NetsshError::SshError(e)
        })?;

        debug!(target: "BaseConnection::open_channel", "Requesting PTY");
        channel.request_pty("xterm", None, None).map_err(|e| {
            info!("Failed to request PTY: {}", e);
            NetsshError::SshError(e)
        })?;

        debug!(target: "BaseConnection::open_channel", "Starting shell");
        channel.shell().map_err(|e| {
            info!("Failed to start shell: {}", e);
            NetsshError::SshError(e)
        })?;

        self.channel = SSHChannel::new(Some(channel));
        debug!(target: "BaseConnection::open_channel", "Successfully opened channel and started shell");
        Ok(())
    }

    pub fn write_channel(&mut self, data: &str) -> Result<(), NetsshError> {
        debug!(target: "BaseConnection::write_channel", "Writing to channel: {:?}", data);

        // Use the SSHChannel to write data
        self.channel.write_channel(data)?;

        // Log the written data if session logging is enabled
        self.session_log.write_raw(data.as_bytes())?;

        debug!(target: "BaseConnection::write_channel", "Successfully wrote to channel");
        Ok(())
    }

    pub fn write_channel_raw(&mut self, data: &[u8]) -> Result<(), NetsshError> {
        debug!(target: "BaseConnection::write_channel_raw", "Writing raw bytes to channel: {:?}", data);

        // Convert bytes to string using the channel's encoding
        let data_str = String::from_utf8_lossy(data).to_string();

        // Use the SSHChannel to write data
        self.channel.write_channel(&data_str)?;

        // Log the written data if session logging is enabled
        self.session_log.write_raw(data)?;

        debug!(target: "BaseConnection::write_channel_raw", "Successfully wrote raw bytes to channel");
        Ok(())
    }

    // Helper method to normalize line feeds (convert \r\n to \n)
    fn normalize_linefeeds(&self, data: &str) -> String {
        // Use the pre-compiled pattern for better performance
        CRLF_PATTERN.replace_all(data, "\n").to_string()
    }

    // Helper method to strip ANSI escape codes
    fn strip_ansi_escape_codes(&self, data: &str) -> String {
        // Use the pre-compiled pattern for better performance
        ANSI_ESCAPE_PATTERN.replace_all(data, "").to_string()
    }

    pub fn read_channel(&mut self) -> Result<String, NetsshError> {
        debug!(target: "BaseConnection::read_channel", "Reading from channel");

        // Create a regex for the prompt if base_prompt is set
        let prompt_regex = if let Some(ref prompt) = self.base_prompt {
            let pattern = format!(r"{}[>#]", regex::escape(prompt));
            match Regex::new(&pattern) {
                Ok(re) => Some(re),
                Err(e) => {
                    debug!(target: "BaseConnection::read_channel", "Failed to create prompt regex: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Use the SSHChannel to read data with the prompt regex
        let new_data = if let Some(re) = prompt_regex.as_ref() {
            debug!(target: "BaseConnection::read_channel", "Reading buffer with prompt regex: {:?}", re);
            self.channel.read_buffer(Some(re))?
        } else {
            debug!(target: "BaseConnection::read_channel", "Reading channel without prompt regex");
            self.channel.read_channel()?
        };

        if self.disable_lf_normalization == false && !new_data.is_empty() {
            // Process line feeds if needed
            let normalized_data = self.normalize_linefeeds(&new_data);

            // Strip ANSI escape codes if configured
            let processed_data = if self.ansi_escape_codes {
                self.strip_ansi_escape_codes(&normalized_data)
            } else {
                normalized_data
            };

            debug!(target: "BaseConnection::read_channel", "Read {} bytes from channel", processed_data.len());

            // Log the data if session logging is enabled
            if self.session_log.is_active() {
                self.session_log.write(&processed_data)?;
            }

            // If data had been previously saved to the buffer, prepend it to output
            if !self._read_buffer.is_empty() {
                let output = self._read_buffer.clone() + &processed_data;
                self._read_buffer.clear();
                return Ok(output);
            }

            return Ok(processed_data);
        }

        // Log the raw data if session logging is enabled and no processing was done
        if self.session_log.is_active() && !new_data.is_empty() {
            self.session_log.write(&new_data)?;
        }

        // If data had been previously saved to the buffer, prepend it to output
        if !self._read_buffer.is_empty() {
            let output = self._read_buffer.clone() + &new_data;
            self._read_buffer.clear();
            return Ok(output);
        }

        Ok(new_data)
    }

    pub fn read_until_pattern(&mut self, pattern: &str) -> Result<String, NetsshError> {
        debug!(target: "BaseConnection::read_until_pattern", "Reading until pattern: {}", pattern);

        // Get current time for timeout tracking
        let start = SystemTime::now();
        
        // Pre-compile the regex pattern for better performance
        let pattern_regex = match Regex::new(pattern) {
            Ok(re) => re,
            Err(e) => {
                debug!(target: "BaseConnection::read_until_pattern", "Invalid regex pattern: {}", e);
                return Err(NetsshError::PatternError(format!("Invalid regex pattern: {}", e)));
            }
        };

        // Use a single, pre-allocated String to avoid multiple allocations during reads
        let mut output = String::with_capacity(16384); // Start with a reasonable capacity
        
        // Use the effective timeout value (override or config default)
        let timeout = self.read_timeout_override.unwrap_or(self.config.read_timeout);
        debug!(target: "BaseConnection::read_until_pattern", "Using timeout: {:?}", timeout);

        loop {
            // Check for timeout
            match start.elapsed() {
                Ok(elapsed) if elapsed > timeout => {
                    debug!(target: "BaseConnection::read_until_pattern", "Timeout reached after {:?}", elapsed);
                    return Err(NetsshError::timeout(format!("finding pattern '{}'", pattern)));
                }
                Ok(_) => {}, // Still within timeout
                Err(e) => return Err(NetsshError::SystemTimeError(e)),
            }

            // Read data from channel
            match self.channel.read_buffer(None) {
                Ok(data) => {
                    if !data.is_empty() {
                        debug!(target: "BaseConnection::read_until_pattern", "Read {} bytes", data.len());
                        
                        // Clean the data if configured
                        let clean_data = if self.ansi_escape_codes {
                            self.strip_ansi_escape_codes(&data)
                        } else {
                            data
                        };

                        // Process line feed normalization if needed
                        let normalized_data = if !self.disable_lf_normalization {
                            self.normalize_linefeeds(&clean_data)
                        } else {
                            clean_data
                        };

                        // Add the new data to our accumulated output
                        output.push_str(&normalized_data);
                        
                        // Log the session data if enabled
                        if self.session_log.is_enabled() {
                            self.session_log.write(&normalized_data)?;
                        }

                        // Check if the pattern exists in the data
                        if pattern_regex.is_match(&output) {
                            debug!(target: "BaseConnection::read_until_pattern", "Found pattern match");
                            break;
                        }
                    }

                    // Sleep a bit to avoid CPU spinning
                    thread::sleep(Duration::from_millis(DEFAULT_LOOP_DELAY_MS));
                }
                Err(e) => {
                    debug!(target: "BaseConnection::read_until_pattern", "Error reading from channel: {}", e);
                    if self.session_log.is_enabled() {
                        self.session_log.write(&format!("Error: {}\n", e))?;
                    }
                    return Err(e);
                }
            }
        }

        debug!(target: "BaseConnection::read_until_pattern", "Read complete, found pattern");
        Ok(output)
    }

    // Special method to handle reading until a prompt (> or #)
    pub fn read_until_prompt(&mut self) -> Result<String, NetsshError> {
        debug!(target: "BaseConnection::read_until_prompt", "Reading until prompt (> or #)");

        let mut output = String::new();
        let loop_delay = Duration::from_millis(100); // 10ms delay between reads
        let start_time = SystemTime::now();

        // Keep reading until timeout or prompt is found
        while start_time.elapsed()? < self.config.pattern_timeout {
            thread::sleep(Duration::from_millis(DEFAULT_LOOP_DELAY_MS));
            // Read a chunk of data
            let new_data = self.read_channel()?;

            debug!(target: "BaseConnection::read_until_prompt", "Read chunk====: {:?}", new_data);

            if !new_data.is_empty() {
                debug!(target: "BaseConnection::read_until_prompt", "Read chunk: {:?}", new_data);
                output.push_str(&new_data);

                // Check for prompt characters at the end of lines
                let lines: Vec<&str> = output.lines().collect();
                if let Some(last_line) = lines.last() {
                    let trimmed = last_line.trim();
                    if trimmed.ends_with(">") || trimmed.ends_with("#") {
                        debug!(target: "BaseConnection::read_until_prompt", "Prompt found: {}", trimmed);
                        return Ok(output);
                    }
                }
            } else {
                // If no new data, try sending a newline to get a prompt
                if output.is_empty() || start_time.elapsed()? > Duration::from_secs(5) {
                    debug!(target: "BaseConnection::read_until_prompt", "Sending newline to get prompt");
                    let _ = self.write_channel("\n");
                }

                // Sleep a bit to avoid CPU spinning
                std::thread::sleep(loop_delay);
            }
        }

        // If we have some output but timed out, return what we have instead of an error
        if !output.is_empty() {
            debug!(target: "BaseConnection::read_until_prompt", "Timeout reached but returning partial output: {:?}", output);
            return Ok(output);
        }

        debug!(target: "BaseConnection::read_until_prompt", "Timeout reached with no output");
        Err(NetsshError::TimeoutError(
            "Prompt not found within timeout period".to_string(),
        ))
    }

    /// Clear any data available in the channel.
    ///
    /// This function reads data from the channel multiple times with a backoff strategy
    /// to ensure all buffered data is cleared.
    ///
    /// # Arguments
    ///
    /// * `backoff` - Whether to increase sleep time when data is detected (default: true)
    /// * `backoff_max` - Maximum sleep time in seconds when using backoff (default: 3.0)
    /// * `delay_factor` - Multiplier for sleep time (default: global_delay_factor or 1.0)
    ///
    /// # Returns
    ///
    /// The accumulated output from the channel
    pub fn clear_buffer(
        &mut self,
        backoff: Option<bool>,
        backoff_max: Option<f64>,
        delay_factor: Option<f64>,
    ) -> Result<String, NetsshError> {
        let backoff = backoff.unwrap_or(true);
        let backoff_max = backoff_max.unwrap_or(3.0);
        let delay_factor = delay_factor.unwrap_or(1.0);

        let mut sleep_time = 0.1 * delay_factor;
        let mut output = String::new();

        info!(
            "Clearing buffer with backoff={}, backoff_max={}, delay_factor={}",
            backoff, backoff_max, delay_factor
        );

        for _ in 0..10 {
            // Sleep before reading
            let sleep_duration = Duration::from_secs_f64(sleep_time);
            thread::sleep(sleep_duration);

            // Read data from channel
            let data = match self.read_channel() {
                Ok(data) => {
                    debug!(target: "BaseConnection::clear_buffer", "Read {} bytes from channel: {:?}", data.len(), data);
                    data
                }
                Err(e) => {
                    warn!(target: "BaseConnection::clear_buffer", "Error reading channel: {}", e);
                    break;
                }
            };

            // Strip ANSI escape codes
            let data = self.strip_ansi_escape_codes(&data);
            debug!(target: "BaseConnection::clear_buffer", "After stripping ANSI codes: {:?}", data);

            // Add to accumulated output
            output.push_str(&data);

            // If no data, we're done
            if data.is_empty() {
                break;
            }

            // Double sleep time if backoff is enabled
            debug!(target: "BaseConnection::clear_buffer", "Clear buffer detects data in the channel");
            if backoff {
                sleep_time *= 2.0;
                if sleep_time > backoff_max {
                    sleep_time = backoff_max;
                }
            }
        }

        debug!(target: "BaseConnection::clear_buffer", "Buffer cleared, accumulated {} bytes", output.len());

        debug!(target: "BaseConnection::clear_buffer", "Buffer cleared, accumulated {} data", output);
        Ok(output)
    }

    pub fn send_command(&mut self, command: &str) -> Result<String, NetsshError> {
        debug!(target: "BaseConnection::send_command", "Sending command: {}", command);

        // Write the command to the channel
        self.write_channel(&format!("{}\n", command))?;

        // Sleep to allow the command to be processed
        self.sleep_for_command(None);

        // Read the response
        let response = self.read_until_prompt()?;
        
        debug!(target: "BaseConnection::send_command", "Command complete, response length: {}", response.len());
        Ok(response)
    }
    
    /// Send multiple commands in a batch to reduce round-trip latency
    pub fn send_commands(&mut self, commands: &[&str]) -> Result<Vec<String>, NetsshError> {
        debug!(target: "BaseConnection::send_commands", "Sending {} commands in batch", commands.len());
        
        let mut responses = Vec::with_capacity(commands.len());
        
        // For single commands, use the standard method
        if commands.len() == 1 {
            let response = self.send_command(commands[0])?;
            responses.push(response);
            return Ok(responses);
        }
        
        // For multiple commands, optimize by sending them all at once
        // with minimal delay between them
        for command in commands {
            debug!(target: "BaseConnection::send_commands", "Writing command: {}", command);
            self.write_channel(&format!("{}\n", command))?;
            
            // Use a shorter delay between commands in batch mode
            thread::sleep(Duration::from_millis(50));
            
            // Read until prompt after each command to capture its response
            let response = self.read_until_prompt()?;
            responses.push(response);
        }
        
        debug!(target: "BaseConnection::send_commands", "Batch command execution complete");
        Ok(responses)
    }
    
    /// Send commands in configuration mode
    pub fn send_config_commands(&mut self, commands: &[&str]) -> Result<Vec<String>, NetsshError> {
        debug!(target: "BaseConnection::send_config_commands", "Sending {} config commands", commands.len());
        
        // This implementation assumes a device that enters config mode with "configure terminal"
        // and exits with "end" - device-specific implementations should override this method
        
        // Enter config mode
        self.send_command("configure terminal")?;
        
        let mut responses = Vec::with_capacity(commands.len());
        
        // Send all configuration commands
        for command in commands {
            let response = self.send_command(command)?;
            responses.push(response);
        }
        
        // Exit config mode
        self.send_command("end")?;
        
        debug!(target: "BaseConnection::send_config_commands", "Config commands execution complete");
        Ok(responses)
    }

    pub fn set_session_log(&mut self, filename: &str) -> Result<(), NetsshError> {
        self.session_log.enable(filename)
    }

    /// Send a keep-alive message to prevent SSH connection timeout
    pub fn keep_alive(&mut self) -> Result<(), NetsshError> {
        debug!(target: "BaseConnection::keep_alive", "Sending keep-alive");
        
        if let Some(ref session) = self.session {
            // Check if the session is still active
            if !session.authenticated() {
                return Err(NetsshError::ConnectionError(
                    "SSH session is no longer authenticated".to_string(),
                ));
            }
            
            // Send an empty command as keep-alive
            match self.write_channel("\n") {
                Ok(_) => {
                    // Discard any output
                    let _ = self.clear_buffer(None, None, None);
                    debug!(target: "BaseConnection::keep_alive", "Keep-alive successful");
                    Ok(())
                },
                Err(e) => {
                    debug!(target: "BaseConnection::keep_alive", "Keep-alive failed: {}", e);
                    Err(e)
                }
            }
        } else {
            Err(NetsshError::ConnectionError(
                "No active SSH session for keep-alive".to_string(),
            ))
        }
    }
    
    /// Start a background keepalive task (for async contexts)
    #[cfg(feature = "async")]
    pub async fn start_keepalive_task(&self) -> Result<tokio::task::JoinHandle<()>, NetsshError> {
        use tokio::time::{interval, Duration as TokioDuration};
        
        // Clone necessary data for the background task
        let host = self.config.host.clone();
        let username = self.config.username.clone();
        let port = self.config.default_port;
        let password = self.config.password.clone();
        
        // Create a new background task for keep-alive
        let handle = tokio::spawn(async move {
            let mut interval = interval(TokioDuration::from_secs(60));
            let mut connection = match BaseConnection::new() {
                Ok(conn) => conn,
                Err(_) => return,
            };
            
            loop {
                interval.tick().await;
                
                // If connection is closed, try to reconnect
                if connection.session.is_none() {
                    if let Err(_) = connection.connect(&host, &username, password.as_deref(), Some(port), None) {
                        // Failed to reconnect, try again next cycle
                        continue;
                    }
                }
                
                // Send keep-alive
                if let Err(_) = connection.keep_alive() {
                    // Keep-alive failed, close and try to reconnect next cycle
                    let _ = connection.close();
                }
            }
        });
        
        Ok(handle)
    }
    
    /// Handle a timeout with exponential backoff retry
    pub fn handle_timeout<F, T>(&self, mut operation: F, max_retries: usize) -> Result<T, NetsshError>
    where 
        F: FnMut() -> Result<T, NetsshError>,
    {
        let mut retry_count = 0;
        let mut backoff_ms = 100; // Start with 100ms backoff
        
        loop {
            match operation() {
                Ok(result) => return Ok(result),
                Err(e) => {
                    // Check if we've hit our retry limit
                    if retry_count >= max_retries {
                        return Err(e);
                    }
                    
                    // Only retry on timeout errors
                    match e {
                        NetsshError::TimeoutError(_) | NetsshError::Timeout { .. } => {
                            // Exponential backoff with jitter
                            let jitter = rand::random::<u64>() % 50;
                            let sleep_time = Duration::from_millis(backoff_ms + jitter);
                            
                            debug!(target: "BaseConnection::handle_timeout", 
                                "Operation timed out, retrying in {:?} (retry {}/{})", 
                                sleep_time, retry_count + 1, max_retries);
                            
                            thread::sleep(sleep_time);
                            retry_count += 1;
                            backoff_ms = std::cmp::min(backoff_ms * 2, 5000); // Cap at 5 seconds
                        },
                        _ => return Err(e), // For non-timeout errors, don't retry
                    }
                }
            }
        }
    }
}
