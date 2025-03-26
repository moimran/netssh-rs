use crate::channel::SSHChannel;
use crate::config::NetsshConfig;
use crate::error::NetsshError;
use crate::patterns::{ANSI_ESCAPE_PATTERN, CRLF_PATTERN};
use crate::session_log::SessionLog;
use log::{debug, info, warn};
use rand;
use regex::Regex;
use ssh2::Session;
use std::net::TcpStream;
use std::thread;
use std::time::{Duration, SystemTime};

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
            }
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

        let id_string = session.banner();

        debug!(target: "BaseConnection::connect ***************", "ID string: {}", id_string.unwrap());

        debug!(target: "BaseConnection::connect", "Authentication successful");
        debug!(target: "BaseConnection::connect", "Opening SSH channel");

        let mut channel = match session.channel_session() {
            Ok(channel) => {
                debug!(target: "BaseConnection::connect", "SSH channel created successfully");
                channel
            }
            Err(e) => {
                info!("Failed to create channel session: {}", e);
                return Err(NetsshError::channel_failed(
                    "Failed to create channel session",
                    Some(e),
                ));
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
        let mut new_data = if let Some(re) = prompt_regex.as_ref() {
            debug!(target: "BaseConnection::read_channel", "Reading buffer with prompt regex: {:?}", re);
            match self.channel.read_buffer(Some(re)) {
                Ok(data) => data,
                Err(e) => {
                    debug!(target: "BaseConnection::read_channel", "Error reading buffer: {}", e);
                    // On timeout, return empty string instead of error
                    if matches!(e, NetsshError::TimeoutError(_)) {
                        String::new()
                    } else {
                        return Err(e);
                    }
                }
            }
        } else {
            debug!(target: "BaseConnection::read_channel", "Reading channel without prompt regex");
            match self.channel.read_channel() {
                Ok(data) => data,
                Err(e) => {
                    debug!(target: "BaseConnection::read_channel", "Error reading channel: {}", e);
                    // On timeout, return empty string instead of error
                    if matches!(e, NetsshError::TimeoutError(_)) {
                        String::new()
                    } else {
                        return Err(e);
                    }
                }
            }
        };

        if !self.disable_lf_normalization && !new_data.is_empty() {
            // Handle data blocks ending in '\r' when '\n' exists
            let start = SystemTime::now();
            while new_data.contains('\n') {
                if let Ok(elapsed) = start.elapsed() {
                    if elapsed > Duration::from_secs(1) {
                        break;
                    }
                }

                if new_data.ends_with('\r') {
                    thread::sleep(Duration::from_millis(10));
                    match self.channel.read_channel() {
                        Ok(data) => new_data.push_str(&data),
                        Err(e) => {
                            if !matches!(e, NetsshError::TimeoutError(_)) {
                                return Err(e);
                            }
                            break;
                        }
                    }
                } else {
                    break;
                }
            }

            // Process line feeds
            new_data = self.normalize_linefeeds(&new_data);
        }

        // Strip ANSI escape codes if configured
        let processed_data = if self.ansi_escape_codes {
            self.strip_ansi_escape_codes(&new_data)
        } else {
            new_data
        };

        debug!(target: "BaseConnection::read_channel", "Read data: {:?}", processed_data);

        // Log the data if session logging is enabled
        if self.session_log.is_active() && !processed_data.is_empty() {
            self.session_log.write(&processed_data)?;
        }

        // If data had been previously saved to the buffer, prepend it to output
        let output = if !self._read_buffer.is_empty() {
            let combined = self._read_buffer.clone() + &processed_data;
            self._read_buffer.clear();
            combined
        } else {
            processed_data
        };

        Ok(output)
    }

    pub fn set_base_prompt(&mut self) -> Result<String, NetsshError> {
        debug!(target: "CiscoBaseConnection::set_base_prompt", "Setting base prompt");

        // Send newline to get prompt
        self.write_channel("\n")?;

        // Wait for a short time to ensure the command is processed
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Read whatever is available
        let pattern = r"[>#]";
        let output = match self.read_until_pattern(pattern, None, None) {
            Ok(out) => out,
            Err(e) => {
                warn!(target: "BaseConnection::set_base_prompt", "Error reading response: {}", e);
                let prompt = "Router".to_string();
                // Also set the base_prompt in the BaseConnection
                self.base_prompt = Some(prompt.clone());
                // Set the prompt in the SSHChannel
                self.channel.set_base_prompt(&prompt);
                return Ok(prompt.clone());
            }
        };

        let prompt = if let Some(last_line) = output
            .lines()
            .filter(|line| line.contains(">") || line.contains("#"))
            .last()
        {
            // Extract the prompt without the terminator
            let prompt_end = last_line
                .find('>')
                .or_else(|| last_line.find('#'))
                .unwrap_or(last_line.len());
            last_line[..prompt_end].trim_end().to_string()
        } else {
            // If we can't find a prompt, use a default
            warn!(target: "CiscoBaseConnection::set_base_prompt", "Could not find prompt in output: {}", output);
            "Router".to_string()
        };

        debug!(target: "CiscoBaseConnection::set_base_prompt", "Base prompt set to: {}", prompt);

        // Also set the base_prompt in the BaseConnection
        self.base_prompt = Some(prompt.clone());
        debug!(target: "CiscoBaseConnection::set_base_prompt", "Set base_prompt in BaseConnection to: {}", prompt);

        // Set the prompt in the SSHChannel
        self.channel.set_base_prompt(&prompt);
        debug!(target: "CiscoBaseConnection::set_base_prompt", "Set base_prompt in SSHChannel");

        Ok(prompt)
    }

    /// Read channel until pattern is detected.
    ///
    /// Will return string up to and including pattern.
    /// Returns TimeoutError if pattern not detected in read_timeout seconds.
    ///
    /// # Arguments
    ///
    /// * `pattern` - Regular expression pattern used to identify that reading is done
    /// * `read_timeout` - Maximum time to wait looking for pattern (default: 10 seconds)
    ///                   A value of 0 means never timeout
    /// * `re_flags` - Regex flags used with pattern (not used in Rust implementation)
    ///
    /// # Returns
    ///
    /// Returns the accumulated output up to and including the pattern
    pub fn read_until_pattern(
        &mut self,
        pattern: &str,
        read_timeout: Option<f64>,
        re_flags: Option<i32>,
    ) -> Result<String, NetsshError> {
        debug!(target: "BaseConnection::read_until_pattern", "Reading until pattern: {}", pattern);

        // Get current time for timeout tracking
        let start = SystemTime::now();

        // Use provided timeout or default to 10 seconds
        let timeout = if let Some(t) = read_timeout {
            if t == 0.0 {
                None // No timeout
            } else {
                Some(Duration::from_secs_f64(t))
            }
        } else {
            Some(Duration::from_secs_f64(10.0)) // Default 10 seconds
        };

        // Pre-compile the regex pattern for better performance
        let pattern_regex = match Regex::new(pattern) {
            Ok(re) => re,
            Err(e) => {
                debug!(target: "BaseConnection::read_until_pattern", "Invalid regex pattern: {}", e);
                return Err(NetsshError::PatternError(format!(
                    "Invalid regex pattern: {}",
                    e
                )));
            }
        };

        // Check for potential issues with pattern
        if pattern.contains('(') && !pattern.contains("(?:") {
            debug!(target: "BaseConnection::read_until_pattern",
                "Warning: Parenthesis found in pattern '{}'. This can be problematic. \
                Consider using non-capture groups '(?:' or wrapping the entire pattern '(pattern)'",
                pattern
            );
        }

        let mut output = String::with_capacity(16384); // Pre-allocate with reasonable size
        let loop_delay = Duration::from_millis(10); // 10ms delay between reads

        loop {
            // Check for timeout if one is set
            if let Some(timeout_duration) = timeout {
                match start.elapsed() {
                    Ok(elapsed) if elapsed > timeout_duration => {
                        let msg = format!(
                            "\n\nPattern not detected: {:?} in output.\n\n\
                            Things you might try to fix this:\n\
                            1. Adjust the regex pattern to better identify the terminating string. Note, in\n\
                            many situations the pattern is automatically based on the network device's prompt.\n\
                            2. Increase the read_timeout to a larger value.\n\n\
                            You can also look at the session log or debug log for more information.\n\n",
                            pattern
                        );
                        debug!(target: "BaseConnection::read_until_pattern", "Timeout reached after {:?}", elapsed);
                        return Err(NetsshError::timeout(msg));
                    }
                    Ok(_) => {} // Still within timeout
                    Err(e) => return Err(NetsshError::SystemTimeError(e)),
                }
            }

            // Read data from channel
            match self.read_channel() {
                Ok(data) => {
                    if !data.is_empty() {
                        debug!(target: "BaseConnection::read_until_pattern", "Read {} bytes", data.len());

                        // Add the new data to our accumulated output
                        output.push_str(&data);

                        // Check if the pattern exists in the accumulated output
                        if pattern_regex.is_match(&output) {
                            debug!(target: "BaseConnection::read_until_pattern", "Found pattern match");

                            // Split the output at the pattern
                            let parts: Vec<&str> =
                                pattern_regex.splitn(output.as_str(), 2).collect();

                            if parts.len() == 2 {
                                // Get everything up to and including the pattern
                                let (before, after) = (parts[0], parts[1]);
                                let pattern_match = pattern_regex
                                    .find(&output)
                                    .map(|m| m.as_str())
                                    .unwrap_or("");

                                // Store any remaining data in the read buffer
                                if !after.is_empty() {
                                    self._read_buffer = after.to_string();
                                }

                                // Return everything up to and including the pattern
                                let result = format!("{}{}", before, pattern_match);
                                debug!(target: "BaseConnection::read_until_pattern", "Returning matched output: {:?}", result);
                                return Ok(result);
                            }
                        }
                    }

                    // Sleep a bit to avoid CPU spinning
                    thread::sleep(loop_delay);
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
    }

    /// Read channel up to and including self.base_prompt.
    ///
    /// # Arguments
    ///
    /// * `read_timeout` - Maximum time to wait looking for prompt (default: 10 seconds)
    /// * `read_entire_line` - If true, reads the entire line containing the prompt
    /// * `re_flags` - Regex flags (not used in Rust implementation)
    ///
    /// # Returns
    ///
    /// Returns the accumulated output up to and including the prompt
    pub fn read_until_prompt(
        &mut self,
        read_timeout: Option<f64>,
        read_entire_line: Option<bool>,
        re_flags: Option<i32>,
    ) -> Result<String, NetsshError> {
        debug!(target: "BaseConnection::read_until_prompt", "Reading until prompt");

        // Get the base prompt or return error if not set
        let base_prompt = self
            .base_prompt
            .as_ref()
            .ok_or_else(|| NetsshError::ConnectionError("Base prompt not set".to_string()))?;

        // Escape the prompt for regex and construct pattern
        let pattern = if read_entire_line.unwrap_or(false) {
            format!("{}.*", regex::escape(base_prompt))
        } else {
            regex::escape(base_prompt)
        };

        debug!(target: "BaseConnection::read_until_prompt",
            "Reading with pattern: {}, timeout: {:?}, read_entire_line: {:?}",
            pattern, read_timeout, read_entire_line
        );

        // Delegate to read_until_pattern
        self.read_until_pattern(&pattern, read_timeout, re_flags)
    }

    /// Helper function to increment delay with a maximum cap
    fn increment_delay(delay: f64, increment: f64, maximum: f64) -> f64 {
        let new_delay = delay * increment;
        if new_delay >= maximum {
            maximum
        } else {
            new_delay
        }
    }

    /// Clear any data available in the channel.
    ///
    /// This function reads data from the channel multiple times with an incrementing delay strategy
    /// to ensure all buffered data is cleared. It will attempt to read data, and if none is found,
    /// it will send a return character and try again with an increased delay.
    ///
    /// # Arguments
    ///
    /// * `pattern` - Optional regex pattern to use for reading (uses read_until_pattern if provided)
    /// * `count` - Maximum number of attempts to read data (default: 40)
    /// * `delay_factor` - Initial delay factor (default: 1.0)
    ///
    /// # Returns
    ///
    /// The accumulated output from the channel
    pub fn clear_buffer(
        &mut self,
        pattern: Option<&str>,
        count: Option<usize>,
        delay_factor: Option<f64>,
    ) -> Result<String, NetsshError> {
        let count = count.unwrap_or(40);
        let delay_factor = delay_factor.unwrap_or(1.0);

        debug!(target: "BaseConnection::clear_buffer",
            "Clearing buffer with pattern={:?}, count={}, delay_factor={}",
            pattern, count, delay_factor
        );

        // If pattern is provided, use read_until_pattern with a 20 second timeout
        if let Some(pattern_str) = pattern {
            debug!(target: "BaseConnection::clear_buffer", "Using pattern-based read");
            return self.read_until_pattern(pattern_str, Some(20.0), None);
        }

        // Initial delay calculation
        let mut main_delay = delay_factor * 0.1;

        // Initial sleep to allow buffer to fill
        thread::sleep(Duration::from_secs_f64(main_delay * 10.0));

        let mut output = String::new();
        let return_str = "\r\n";

        for i in 0..=count {
            debug!(target: "BaseConnection::clear_buffer", "Read attempt {} with delay {}", i, main_delay);

            // Try to read data with timing
            match self.read_channel_timing(None, Some(20.0)) {
                Ok(new_data) => {
                    if !new_data.is_empty() {
                        debug!(target: "BaseConnection::clear_buffer", "Received data on attempt {}", i);
                        output.push_str(&new_data);
                        return Ok(output);
                    }
                }
                Err(e) => {
                    warn!(target: "BaseConnection::clear_buffer", "Error reading channel: {}", e);
                    // Continue trying rather than breaking on error
                }
            }

            // If we haven't returned yet, write return character and try again
            debug!(target: "BaseConnection::clear_buffer", "No data, sending return character");
            if let Err(e) = self.write_channel(return_str) {
                warn!(target: "BaseConnection::clear_buffer", "Error writing return: {}", e);
            }

            // Increment delay and sleep
            main_delay = Self::increment_delay(main_delay, 1.1, 8.0);
            thread::sleep(Duration::from_secs_f64(main_delay));
        }

        // If we've exhausted all attempts without getting data
        Err(NetsshError::TimeoutError(
            "Timed out waiting for data".to_string(),
        ))
    }

    pub fn send_command(&mut self, command: &str) -> Result<String, NetsshError> {
        debug!(target: "BaseConnection::send_command", "Sending command: {}", command);

        // Write the command to the channel
        self.write_channel(&format!("{}\n", command))?;

        // Sleep to allow the command to be processed
        self.sleep_for_command(None);

        // Read the response
        let response = self.read_until_prompt(None, None, None)?;

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
            let response = self.read_until_prompt(None, None, None)?;
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
                }
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
                    if let Err(_) =
                        connection.connect(&host, &username, password.as_deref(), Some(port), None)
                    {
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
    pub fn handle_timeout<F, T>(
        &self,
        mut operation: F,
        max_retries: usize,
    ) -> Result<T, NetsshError>
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
                        }
                        _ => return Err(e), // For non-timeout errors, don't retry
                    }
                }
            }
        }
    }

    /// Get the SSH server's remote version string
    pub fn get_remote_version(&self) -> Option<String> {
        if let Some(session) = &self.session {
            // The banner() method returns Option<&str>, not Result
            if let Some(banner) = session.banner() {
                return Some(banner.to_string());
            }
        }
        None
    }

    /// Read data on the channel based on timing delays.
    ///
    /// General pattern is keep reading until no new data is read.
    /// Once no new data is read wait `last_read` amount of time (one last read).
    /// As long as no new data, then return data.
    ///
    /// Setting `read_timeout` to zero will cause read_channel_timing to never expire based
    /// on an absolute timeout. It will only complete based on timeout based on there being
    /// no new data.
    ///
    /// # Arguments
    ///
    /// * `last_read` - Amount of time in seconds to wait before performing one last read
    /// * `read_timeout` - Absolute timer in seconds for how long to keep reading. Zero means never timeout.
    ///
    /// # Returns
    ///
    /// Returns the accumulated channel data as a String
    pub fn read_channel_timing(
        &mut self,
        last_read: Option<f64>,
        read_timeout: Option<f64>,
    ) -> Result<String, NetsshError> {
        let last_read = Duration::from_secs_f64(last_read.unwrap_or(2.0));
        let read_timeout = if let Some(timeout) = read_timeout {
            if timeout == 0.0 {
                None // No timeout
            } else {
                Some(Duration::from_secs_f64(timeout))
            }
        } else {
            Some(Duration::from_secs_f64(120.0)) // Default 120 seconds
        };

        // Time to delay in each read loop
        let loop_delay = Duration::from_millis(100);
        let mut channel_data = String::new();
        let start_time = SystemTime::now();

        loop {
            // Check if we've exceeded read_timeout (if one is set)
            if let Some(timeout) = read_timeout {
                if let Ok(elapsed) = start_time.elapsed() {
                    if elapsed > timeout {
                        let msg = format!(
                            "\nread_channel_timing's absolute timer expired.\n\n\
                            The network device was continually outputting data for longer than {} \
                            seconds.\n\n\
                            If this is expected i.e. the command you are executing is continually emitting \
                            data for a long period of time, then you can set 'read_timeout=x' seconds. If \
                            you want to keep reading indefinitely (i.e. to only stop when there is \
                            no new data), then you can set 'read_timeout=0'.\n\n\
                            You can look at the session_log or debug log for more information.\n",
                            timeout.as_secs()
                        );
                        return Err(NetsshError::TimeoutError(msg));
                    }
                }
            }

            // Sleep before reading
            thread::sleep(loop_delay);

            // Read new data
            let new_data = self.read_channel()?;

            // If we have new data, add it
            if !new_data.is_empty() {
                channel_data.push_str(&new_data);
            }
            // If we have some output, but nothing new, then do the last read
            else if !channel_data.is_empty() {
                // Make sure really done (i.e. no new data)
                thread::sleep(last_read);
                let final_data = self.read_channel()?;
                if !final_data.is_empty() {
                    channel_data.push_str(&final_data);
                } else {
                    break;
                }
            }
        }

        Ok(channel_data)
    }
}
