use crate::channel::SSHChannel;
use crate::config::NetsshConfig;
use crate::device_connection::DeviceType;
use crate::error::NetsshError;
use crate::patterns::{ANSI_ESCAPE_PATTERN, CRLF_PATTERN};
use crate::session_log::SessionLog;
use crate::vendor_error_patterns;
use rand;
use regex::Regex;
use ssh2::Session;
use std::collections::VecDeque;
use std::net::TcpStream;
use std::thread;
use std::time::{Duration, SystemTime};
use tracing::{debug, info, instrument, warn};

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
    pub device_type: DeviceType,
}

// Constants for sleep durations
const DEFAULT_COMMAND_WAIT_MS: u64 = 500; // Used by sleep_for_command
                                          // const DEFAULT_LOOP_DELAY_MS: u64 = 10;  // Kept for potential future use

impl BaseConnection {
    // Helper method to sleep for a configurable duration
    // Kept for potential future use
    #[allow(dead_code)]
    fn sleep_for_command(&self, duration_ms: Option<u64>) {
        let duration = Duration::from_millis(duration_ms.unwrap_or(DEFAULT_COMMAND_WAIT_MS));
        debug!(target: "BaseConnection::sleep_for_command", "Sleeping for {:?}", duration);
        thread::sleep(duration);
    }

    // #[instrument(level = "debug")]
    pub fn new() -> Result<Self, NetsshError> {
        debug!("Creating new base connection");
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
            device_type: DeviceType::Unknown,
        })
    }

    // #[instrument(fields(config = ?std::any::type_name::<NetsshConfig>()), level = "debug")]
    pub fn with_config(config: NetsshConfig) -> Result<Self, NetsshError> {
        debug!("Creating base connection with custom config");
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
            device_type: DeviceType::Unknown,
        })
    }

    // #[instrument(skip(self), level = "debug", name = "BaseConnection::connect")]
    pub fn connect(
        &mut self,
        host: Option<&str>,
        username: Option<&str>,
        password: Option<&str>,
        port: Option<u16>,
        connect_timeout: Option<Duration>,
    ) -> Result<(), NetsshError> {
        // Use provided parameters or fall back to config values
        let host = if let Some(h) = host {
            h
        } else if !self.config.host.is_empty() {
            &self.config.host
        } else {
            return Err(NetsshError::ConnectionError(
                "No host specified".to_string(),
            ));
        };

        let username = if let Some(u) = username {
            u
        } else if !self.config.username.is_empty() {
            &self.config.username
        } else {
            return Err(NetsshError::ConnectionError(
                "No username specified".to_string(),
            ));
        };

        // Use provided password or fall back to config
        let password = if let Some(pass) = password {
            Some(pass)
        } else {
            self.config.password.as_deref()
        };

        debug!(
            "Connecting to {}:{}",
            host,
            port.unwrap_or(self.config.default_port)
        );

        let port = port.unwrap_or(self.config.default_port);
        let timeout = connect_timeout.unwrap_or(self.config.connection_timeout);

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

        // Configure SSH session with settings from settings.json

        // Convert timeout from seconds to milliseconds (ssh2 expects milliseconds)
        // let timeout_ms = self.config.blocking_timeout.as_millis() as u32;
        // debug!(target: "BaseConnection::connect", "Setting blocking timeout to {:?} milliseconds", timeout_ms);
        // session.set_timeout(1000);

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

        // After successful connection, enable keepalive mechanism

        debug!(target: "BaseConnection::connect", "Enabling SSH keep-alive");
        session.set_keepalive(true, 60); // Send keepalive every 60 seconds
        session.set_blocking(true);
        // Store the session
        self.session = Some(session);
        self.channel = SSHChannel::new(Some(channel));

        debug!(target: "BaseConnection::connect", "Connection established successfully");
        Ok(())
    }

    // #[instrument(skip(self), level = "debug", name = "BaseConnection::open_channel")]
    pub fn open_channel(&mut self) -> Result<(), NetsshError> {
        debug!("Opening SSH channel");
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

    // #[instrument(
    //     skip(self, data),
    //     level = "debug",
    //     name = "BaseConnection::write_channel"
    // )]
    pub fn write_channel(&mut self, data: &str) -> Result<(), NetsshError> {
        debug!("Writing to channel: {:?}", data);

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

    // #[instrument(skip_all, level = "debug", name = "BaseConnection::read_channel")]
    pub fn read_channel(&mut self) -> Result<String, NetsshError> {
        debug!("Reading from channel");

        // Use the SSHChannel to read data with the prompt regex
        let mut new_data = self.channel.read_channel()?;

        debug!(target: "BaseConnection::read_channel", "disable_lf_normalization: {}, Read data: {:?},", self.disable_lf_normalization, new_data);

        if !self.disable_lf_normalization && !new_data.is_empty() {
            // Handle data blocks ending in '\r' when '\n' exists
            // Data blocks shouldn't end in '\r' (can cause problems with normalize_linefeeds)
            // Only do the extra read if '\n' exists in the output
            // This avoids devices that only use \r
            let start = SystemTime::now();
            while new_data.contains('\n') && start.elapsed().unwrap_or_default().as_secs_f32() < 1.0
            {
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

        // debug!(target: "BaseConnection::read_channel", "Read data: {:?}", processed_data);

        // Log the data if session logging is enabled
        if self.session_log.is_active() && !processed_data.is_empty() {
            self.session_log.write(&processed_data)?;
        }

        debug!(target: "BaseConnection::read_channel", "Processed data: {:?}", processed_data);

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

    /// Sets self.base_prompt
    ///
    /// Used as delimiter for stripping of trailing prompt in output.
    ///
    /// Should be set to something that is general and applies in multiple contexts. For Cisco
    /// devices this will be set to router hostname (i.e. prompt without > or #).
    ///
    /// This will be set on entering user exec or privileged exec on Cisco, but not when
    /// entering/exiting config mode.
    ///
    /// # Arguments
    ///
    /// * `pri_prompt_terminator` - Primary trailing delimiter for identifying a device prompt
    /// * `alt_prompt_terminator` - Alternate trailing delimiter for identifying a device prompt
    /// * `delay_factor` - See __init__: global_delay_factor
    /// * `pattern` - Regular expression pattern to search for in find_prompt() call
    pub fn set_base_prompt(
        &mut self,
        pri_prompt_terminator: Option<&str>,
        alt_prompt_terminator: Option<&str>,
        delay_factor: Option<f64>,
        pattern: Option<&str>,
    ) -> Result<String, NetsshError> {
        // add debug with all the arguments
        debug!(
            "Setting base prompt with arguments: {:?}, {:?}, {:?}, {:?}",
            pri_prompt_terminator, alt_prompt_terminator, delay_factor, pattern
        );
        let pri_prompt_terminator = pri_prompt_terminator.unwrap_or("#");
        let alt_prompt_terminator = alt_prompt_terminator.unwrap_or(">");

        // Build pattern based on terminators if not provided
        let search_pattern = if pattern.is_none() {
            if !pri_prompt_terminator.is_empty() && !alt_prompt_terminator.is_empty() {
                let _pri_term = regex::escape(pri_prompt_terminator);
                let _alt_term = regex::escape(alt_prompt_terminator);
                format!(r"[{}{}]", pri_prompt_terminator, alt_prompt_terminator)
            } else if !pri_prompt_terminator.is_empty() {
                regex::escape(pri_prompt_terminator)
            } else if !alt_prompt_terminator.is_empty() {
                regex::escape(alt_prompt_terminator)
            } else {
                String::new()
            }
        } else {
            pattern.unwrap().to_string()
        };

        debug!("Search pattern: {}", search_pattern);

        // Get the prompt using find_prompt
        let prompt = if !search_pattern.is_empty() {
            self.find_prompt(delay_factor, Some(&search_pattern))?
        } else {
            self.find_prompt(delay_factor, None)?
        };
        // trim the prompt
        let prompt = prompt.trim();

        debug!(target: "BaseConnection::set_base_prompt", "Prompt found by find_prompt: {}", prompt);

        // Verify the prompt ends with a terminator
        if !prompt.chars().last().map_or(false, |c| {
            c.to_string() == pri_prompt_terminator || c.to_string() == alt_prompt_terminator
        }) {
            return Err(NetsshError::PromptError(format!(
                "Router prompt not found: {}",
                prompt
            )));
        }

        // If all we have is the terminator just use that
        if prompt.len() == 1 {
            self.base_prompt = Some(prompt.to_string());
            return Ok(prompt.to_string());
        }

        // Strip off trailing terminator from prompt
        self.base_prompt = Some(prompt[..prompt.len() - 1].to_string());

        debug!(target: "BaseConnection::set_base_prompt", "Base prompt set to: {}", self.base_prompt.clone().unwrap());

        Ok(self.base_prompt.clone().unwrap())
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
        debug!(target: "BaseConnection::read_until_pattern", "Reading until pattern: {} and timeout: {:?} re_flags: {:?}", pattern, read_timeout, re_flags);

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

        debug!(target: "BaseConnection::read_until_pattern", "Pattern regex: {} pattern: {} timeout: {:?}", pattern_regex, pattern, timeout);

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
                                                    // Get current time for timeout tracking
        let mut start = SystemTime::now();

        loop {
            // Check for timeout if one is set
            if let Some(timeout_duration) = timeout {
                match start.elapsed() {
                    Ok(elapsed) if elapsed > timeout_duration => {
                        debug!(target: "BaseConnection::read_until_pattern", "Timeout reached after {:?} timeout: {:?}", elapsed, timeout_duration);
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
                        return Err(NetsshError::TimeoutError(msg));
                    }
                    Ok(_) => {} // Still within timeout
                    Err(e) => {
                        debug!(target: "BaseConnection::read_until_pattern", "System time error: {:?}", e);
                        // Rather than failing, we'll continue reading
                        // This can happen when system clock is adjusted during operation
                        debug!(target: "BaseConnection::read_until_pattern", "Resetting timeout start time and continuing");
                        // Reset the start time to now to avoid repeated errors
                        start = SystemTime::now();
                    }
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
                            debug!(target: "BaseConnection::read_until_pattern", "Found pattern match pattern: {}", pattern_regex);

                            debug!(target: "BaseConnection::read_until_pattern", "Output: {}", output);

                            // Split the output at the pattern
                            // let parts: Vec<&str> =
                            //     pattern_regex.splitn(output.as_str(), 2).collect();

                            // if parts.len() == 2 {
                            //     // Get everything up to and including the pattern
                            //     let (before, after) = (parts[0], parts[1]);
                            //     let pattern_match = pattern_regex
                            //         .find(&output)
                            //         .map(|m| m.as_str())
                            //         .unwrap_or("");

                            //     // Store any remaining data in the read buffer
                            //     if !after.is_empty() {
                            //         self._read_buffer = after.to_string();
                            //     }

                            //     // Return everything up to and including the pattern
                            //     let result = format!("{}{}", before, pattern_match);
                            //     debug!(target: "BaseConnection::read_until_pattern", "Returning matched output: {:?}", result);
                            //     return Ok(result);
                            // }
                            return Ok(output);
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
            format!("{}.*", base_prompt.trim())
        } else {
            regex::escape(base_prompt.trim())
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
    /// * `delay_factor` - Multiplier to adjust delays (default: 1.0)
    /// * `pattern` - Optional regex pattern to use for reading (uses read_until_pattern if provided)
    /// * `max_loops` - Maximum number of attempts to read data (default: 150)
    /// * `initial_delay` - Initial delay before first read (default: 0.1)
    /// * `loop_delay` - Delay between read attempts (default: 0.1)
    /// * `max_delay` - Maximum delay between read attempts (default: 1.0)
    ///
    /// # Returns
    ///
    /// The accumulated output from the channel
    pub fn clear_buffer(
        &mut self,
        delay_factor: Option<f64>,
        pattern: Option<&str>,
        max_loops: Option<usize>,
        initial_delay: Option<f64>,
        loop_delay: Option<f64>,
        max_delay: Option<f64>,
    ) -> Result<String, NetsshError> {
        let delay_factor = delay_factor.unwrap_or(1.0);
        let max_loops = max_loops.unwrap_or(10);
        let initial_delay = initial_delay.unwrap_or(0.1);
        let loop_delay = loop_delay.unwrap_or(0.1);
        let max_delay = max_delay.unwrap_or(1.0);

        debug!(target: "BaseConnection::clear_buffer",
            "Clearing buffer with pattern={:?}, delay_factor={}, max_loops={}, initial_delay={}, loop_delay={}, max_delay={}",
            pattern, delay_factor, max_loops, initial_delay, loop_delay, max_delay
        );

        // If pattern is provided, use read_until_pattern with a 20 second timeout
        if let Some(pattern_str) = pattern {
            debug!(target: "BaseConnection::clear_buffer", "Using pattern-based read");
            return self.read_until_pattern(pattern_str, None, None);
        }

        // Initial sleep to allow buffer to fill
        let initial_sleep = initial_delay * delay_factor;
        thread::sleep(Duration::from_secs_f64(initial_sleep));

        let mut output = String::new();
        let mut current_delay = loop_delay * delay_factor;
        let mut i = 1;

        while i <= max_loops {
            debug!(target: "BaseConnection::clear_buffer", "Read attempt {} with delay {}", i, current_delay);

            // Try to read data
            match self.read_channel() {
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
            if let Err(e) = self.write_channel("\r\n") {
                warn!(target: "BaseConnection::clear_buffer", "Error writing return: {}", e);
            }

            // Increment delay using exponential backoff with a maximum cap
            current_delay = (current_delay * 1.1).min(max_delay);
            thread::sleep(Duration::from_secs_f64(current_delay));

            i += 1;
        }

        // If we've exhausted all attempts without getting data
        Err(NetsshError::TimeoutError(
            "Timed out waiting for data".to_string(),
        ))
    }

    /// Sanitize the output by removing command echo and trailing router prompt.
    ///
    /// # Arguments
    ///
    /// * `output` - The output string to be sanitized
    /// * `strip_command` - Whether to remove command echo from output
    /// * `command_string` - Command string to be removed
    /// * `strip_prompt` - Whether to remove trailing prompt
    ///
    /// # Returns
    ///
    /// Sanitized output string
    pub fn _sanitize_output(
        &self,
        output: &str,
        strip_command: bool,
        command_string: Option<&str>,
        strip_prompt: bool,
    ) -> String {
        let mut result = output.to_string();

        // Strip command echo if requested and command is provided
        if strip_command && command_string.is_some() {
            result = self.strip_command(command_string.unwrap(), &result);
        }

        // Strip prompt if requested
        if strip_prompt {
            result = self.strip_prompt(&result);
        }

        result
    }

    /// Strip command echo from the output.
    ///
    /// # Arguments
    ///
    /// * `command_string` - Command string to be removed
    /// * `output` - Output string containing the command echo
    ///
    /// # Returns
    ///
    /// Output with command echo removed
    pub fn strip_command(&self, command_string: &str, output: &str) -> String {
        let cmd = command_string.trim();

        // If output starts with the command, remove it
        if output.starts_with(cmd) {
            // Split by command and take everything after it
            let parts: Vec<&str> = output.split(cmd).collect();
            if parts.len() >= 2 {
                // Join everything after the command
                return parts[1..].join(cmd);
            }
        }

        // If no match, return original output
        output.to_string()
    }

    /// Strip the trailing router prompt from the output.
    ///
    /// # Arguments
    ///
    /// * `a_string` - String returned from device
    ///
    /// # Returns
    ///
    /// String with trailing prompt removed
    pub fn strip_prompt(&self, a_string: &str) -> String {
        // Get base prompt if available
        if let Some(base_prompt) = &self.base_prompt {
            // Split by newline
            let response_list: Vec<&str> = a_string.split('\n').collect();

            if !response_list.is_empty() {
                let last_line = response_list[response_list.len() - 1];

                // Check if last line contains the prompt
                if last_line.contains(base_prompt) {
                    // Join all lines except the last one
                    return response_list[0..response_list.len() - 1].join("\n");
                }
            }
        }

        // If no prompt or not found, return original string
        a_string.to_string()
    }

    pub fn command_echo_read(
        &mut self,
        command_string: &str,
        read_timeout: f64,
    ) -> Result<String, NetsshError> {
        let cmd_to_verify = command_string.trim();
        let mut output = String::new();

        // Implement command echo verification logic
        if !cmd_to_verify.is_empty() {
            debug!(target: "BaseConnection::command_echo_read", "Verifying command echo and reading response: {}", cmd_to_verify);
            // Escape the command string before using it as a regex pattern
            let escaped_cmd = regex::escape(cmd_to_verify);
            debug!(target: "BaseConnection::command_echo_read", "Original command: '{}', Escaped command: '{}'", cmd_to_verify, escaped_cmd);
            match self.read_until_pattern(&escaped_cmd, Some(read_timeout), None) {
                Ok(echo_data) => {
                    output.push_str(&echo_data);
                    debug!(target: "BaseConnection::command_echo_read", "Command echo verified: {}", echo_data);
                }
                Err(e) => {
                    debug!(target: "BaseConnection::command_echo_read", "Command echo verification failed: {}", e);
                    // Continue anyway, but note the failure
                }
            }
        }

        Ok(output)
    }

    pub fn send_command(
        &mut self,
        command_string: &str,
        expect_string: Option<&str>,
        read_timeout: Option<f64>,
        auto_find_prompt: Option<bool>,
        strip_prompt: Option<bool>,
        strip_command: Option<bool>,
        normalize: Option<bool>,
        cmd_verify: Option<bool>,
    ) -> Result<String, NetsshError> {
        debug!("Sending command: {}", command_string);

        // Default values
        let read_timeout = read_timeout.unwrap_or(10.0);
        let auto_find_prompt = auto_find_prompt.unwrap_or(true);
        let strip_prompt = strip_prompt.unwrap_or(true);
        let strip_command = strip_command.unwrap_or(true);
        let normalize = normalize.unwrap_or(true);
        let cmd_verify = cmd_verify.unwrap_or(true);

        debug!(
            target: "BaseConnection::send_command","========================================="
        );
        debug!(target: "BaseConnection::send_command", "read_timeout: {}", read_timeout);
        debug!(target: "BaseConnection::send_command", "auto_find_prompt: {}", auto_find_prompt);
        debug!(target: "BaseConnection::send_command", "strip_prompt: {}", strip_prompt);
        debug!(target: "BaseConnection::send_command", "strip_command: {}", strip_command);
        debug!(target: "BaseConnection::send_command", "normalize: {}", normalize);
        debug!(target: "BaseConnection::send_command", "cmd_verify: {}", cmd_verify);
        debug!(target: "BaseConnection::send_command", "command_string: {}", command_string);
        debug!(target: "BaseConnection::send_command", "expect_string: {:?}", expect_string);

        // Get search pattern from expect_string or from prompt
        let search_pattern = if let Some(pattern) = expect_string {
            pattern.to_string()
        } else {
            // Handle auto_find_prompt
            if auto_find_prompt {
                match self.find_prompt(None, None) {
                    Ok(prompt) => regex::escape(&prompt),
                    Err(_) => {
                        if let Some(ref base_prompt) = self.base_prompt {
                            regex::escape(base_prompt)
                        } else {
                            return Err(NetsshError::PromptError(
                                "No prompt found for pattern".to_string(),
                            ));
                        }
                    }
                }
            } else if let Some(ref base_prompt) = self.base_prompt {
                regex::escape(base_prompt)
            } else {
                return Err(NetsshError::PromptError("No base prompt set".to_string()));
            }
        };

        // Normalize command if needed
        let cmd = if normalize {
            format!("{}\n", command_string.trim())
        } else {
            command_string.to_string()
        };

        // Start timing for timeout calculation
        let start_time = SystemTime::now();

        // Write command to channel
        self.write_channel(&cmd)?;

        let mut output = String::new();

        // Command verification - make sure we see our command echoed back
        if cmd_verify {
            // Call the extracted method for command echo verification
            match self.command_echo_read(command_string, 10.0) {
                Ok(echo_data) => {
                    output.push_str(&echo_data);
                }
                Err(e) => {
                    debug!(target: "BaseConnection::send_command", "Command echo verification failed: {}", e);
                    // Continue anyway, but note the failure
                }
            }
        }

        // Read until we find the specified pattern - using a deque approach for large outputs
        debug!(target: "BaseConnection::send_command", "Reading until pattern: {}", search_pattern);

        const MAX_CHARS: usize = 2_000_000;
        const DEQUE_SIZE: usize = 20;
        let mut past_n_reads: VecDeque<String> = VecDeque::with_capacity(DEQUE_SIZE);
        let mut first_line_processed = false;

        debug!(target: "BaseConnection::send_command", "======output: {}", output);

        // Process existing output before reading from channel
        if !output.is_empty() {
            past_n_reads.push_back(output.clone());

            // Process first line right away if needed
            if !first_line_processed {
                let (processed_output, processed) =
                    self.first_line_handler(&output, &search_pattern);
                output = processed_output;
                first_line_processed = processed;

                // Check if we already have the pattern in the output
                if let Ok(re) = Regex::new(&search_pattern) {
                    if re.is_match(&output) {
                        debug!(target: "BaseConnection::send_command", "Pattern found in initial output");
                        // Skip reading loop since we already found the pattern
                        // Sanitize output
                        let sanitized_output = self._sanitize_output(
                            &output,
                            strip_command,
                            Some(command_string),
                            strip_prompt,
                        );

                        debug!(target: "BaseConnection::send_command", "Command complete, response length: {}", sanitized_output.len());
                        // Check for device-specific error patterns
                        if let Err(err) =
                            vendor_error_patterns::check_command_output(&sanitized_output, &self.device_type)
                        {
                            debug!("Command produced error pattern: {}", err);
                            return Err(err);
                        }
                        return Ok(sanitized_output);
                    }
                }
            }
        }

        // Keep reading data until search_pattern is found or until read_timeout
        let loop_delay = 0.025; // seconds
        while SystemTime::now()
            .duration_since(start_time)
            .unwrap_or_default()
            .as_secs_f64()
            < read_timeout
        {
            // First check if we have the pattern in existing output
            if !first_line_processed {
                let (processed_output, processed) =
                    self.first_line_handler(&output, &search_pattern);
                output = processed_output;
                first_line_processed = processed;

                // Check if we have already found our pattern
                if let Ok(re) = Regex::new(&search_pattern) {
                    if re.is_match(&output) {
                        debug!(target: "BaseConnection::send_command", "Pattern found after first_line_handler");
                        break;
                    }
                }
            } else {
                if output.len() <= MAX_CHARS {
                    // For smaller outputs, search the entire output
                    if let Ok(re) = Regex::new(&search_pattern) {
                        if re.is_match(&output) {
                            debug!(target: "BaseConnection::send_command", "Pattern found in entire output");
                            break;
                        }
                    }
                } else {
                    // For larger outputs, only search in the recent reads
                    let recent_data: String = past_n_reads.iter().cloned().collect();
                    if let Ok(re) = Regex::new(&search_pattern) {
                        if re.is_match(&recent_data) {
                            debug!(target: "BaseConnection::send_command", "Pattern found in recent reads");
                            break;
                        }
                    }
                }
            }

            // Now try to read more data only if we haven't found the pattern yet
            match self.read_channel() {
                Ok(new_data) => {
                    if !new_data.is_empty() {
                        output.push_str(&new_data);
                        past_n_reads.push_back(new_data);
                        if past_n_reads.len() > DEQUE_SIZE {
                            past_n_reads.pop_front();
                        }
                    }
                }
                Err(e) => {
                    debug!(target: "BaseConnection::send_command", "Error reading channel: {}", e);
                    // Continue anyway, might just be a temporary issue
                }
            }

            // Sleep for a short time before trying again
            thread::sleep(Duration::from_secs_f64(loop_delay));
        }

        // Check if we timed out
        if SystemTime::now()
            .duration_since(start_time)
            .unwrap_or_default()
            .as_secs_f64()
            >= read_timeout
        {
            let msg = format!(
                "Pattern not detected: {:?} in output.\n\nThings you might try to fix this:\n1. Explicitly set your pattern using the expect_string argument.\n2. Increase the read_timeout to a larger value.\n\nYou can also look at the session log or debug log for more information.\n",
                search_pattern
            );
            return Err(NetsshError::TimeoutError(msg));
        }

        // Sanitize output
        let sanitized_output =
            self._sanitize_output(&output, strip_command, Some(command_string), strip_prompt);

        debug!(target: "BaseConnection::send_command", "Command complete, response length: {}", sanitized_output.len());

        // Check for device-specific error patterns
        if let Err(err) =
            vendor_error_patterns::check_command_output(&sanitized_output, &self.device_type)
        {
            debug!("Command produced error pattern: {}", err);
            return Err(err);
        }

        // Return the output if no error patterns were found
        Ok(sanitized_output)
    }

    /// Send multiple commands in a batch to reduce round-trip latency
    pub fn send_commands(&mut self, commands: &[&str]) -> Result<Vec<String>, NetsshError> {
        debug!(target: "BaseConnection::send_commands", "Sending {} commands in batch", commands.len());

        let mut responses = Vec::with_capacity(commands.len());

        // For single commands, use the standard method
        if commands.len() == 1 {
            let response =
                self.send_command(commands[0], None, None, None, None, None, None, None)?;
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
                    let _ = self.clear_buffer(None, None, None, None, None, None);
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
        let _last_read = Duration::from_secs_f64(last_read.unwrap_or(2.0));
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

        debug!(target: "BaseConnection::read_channel_timing", "Reading channel with read_timeout: {:?}, loop_delay: {:?}", read_timeout, loop_delay);

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
                break;
            }
            // If we have some output, but nothing new, then do the last read
            // else if !channel_data.is_empty() {
            //     // Make sure really done (i.e. no new data)
            //     thread::sleep(last_read);
            //     let final_data = self.read_channel()?;
            //     if !final_data.is_empty() {
            //         channel_data.push_str(&final_data);
            //     } else {
            //         break;
            //     }
            // }
        }

        Ok(channel_data)
    }

    #[instrument(skip(self), level = "debug", name = "BaseConnection::find_prompt")]
    pub fn find_prompt(
        &mut self,
        delay_factor: Option<f64>,
        pattern: Option<&str>,
    ) -> Result<String, NetsshError> {
        debug!(
            "Finding prompt with delay_factor: {:?}, pattern: {:?}",
            delay_factor, pattern
        );

        // Clear the buffer
        // self.clear_buffer(None, Some(r"[>#]"), None, None, None, None)?;

        // Send newline to get prompt
        self.write_channel("\n")?;

        // Calculate sleep time based on delay factor
        let sleep_time = Duration::from_secs_f64(delay_factor.unwrap_or(1.0) * 0.25);
        thread::sleep(sleep_time);

        // Read the response
        let output = if let Some(pat) = pattern {
            debug!(target: "BaseConnection::find_prompt", "Reading until pattern: {}", pat);
            match self.read_until_pattern(pat, None, None) {
                Ok(data) => data,
                Err(e) => {
                    warn!(target: "BaseConnection::find_prompt", "Error reading response: {}", e);
                    return Err(e);
                }
            }
        } else {
            // Initial read
            match self.read_channel() {
                Ok(data) => {
                    let mut prompt = data.trim().to_string();
                    let mut count = 0;
                    let mut current_sleep_time = sleep_time;

                    debug!(target: "BaseConnection::find_prompt", "Initial read result: {:?}", prompt);

                    // If we don't get a prompt, try a few more times
                    while count <= 12 && prompt.is_empty() {
                        if prompt.is_empty() {
                            debug!(target: "BaseConnection::find_prompt", "No prompt found, sending newline (attempt {})", count + 1);
                            self.write_channel("\n")?;
                            thread::sleep(current_sleep_time);
                            match self.read_channel() {
                                Ok(new_data) => prompt = new_data.trim().to_string(),
                                Err(e) => {
                                    if matches!(e, NetsshError::TimeoutError(_)) {
                                        debug!(target: "BaseConnection::find_prompt", "Timeout reading channel, continuing");
                                        // On timeout, just continue the loop
                                    } else {
                                        return Err(e);
                                    }
                                }
                            }

                            // Increase sleep time with each loop
                            if current_sleep_time.as_secs_f64() <= 3.0 {
                                // Double the sleep_time when it is small
                                current_sleep_time =
                                    Duration::from_secs_f64(current_sleep_time.as_secs_f64() * 2.0);
                                debug!(target: "BaseConnection::find_prompt", "Doubled sleep time to {:?}", current_sleep_time);
                            } else {
                                current_sleep_time =
                                    Duration::from_secs_f64(current_sleep_time.as_secs_f64() + 1.0);
                                debug!(target: "BaseConnection::find_prompt", "Increased sleep time to {:?}", current_sleep_time);
                            }
                        }
                        count += 1;
                    }
                    prompt
                }
                Err(e) => {
                    warn!(target: "BaseConnection::find_prompt", "Error reading channel: {}", e);
                    return Err(e);
                }
            }
        };

        let prompt = output;

        // Process the output to extract the prompt
        // let prompt = if let Some(last_line) = output
        //     .lines()
        //     .filter(|line| line.contains(">") || line.contains("#"))
        //     .last()
        // {
        //     // Extract the prompt without the terminator
        //     let prompt_end = last_line
        //         .find('>')
        //         .or_else(|| last_line.find('#'))
        //         .unwrap_or(last_line.len());
        //     last_line[..prompt_end].trim_end().to_string()
        // } else if output.contains('\n') {
        //     // If no line has a prompt character, just use the last line
        //     output.lines().last().unwrap_or("").trim().to_string()
        // } else {
        //     // Single line output
        //     output.trim().to_string()
        // };

        // Clear the buffer again
        // self.clear_buffer(None, None, None, None, None, None)?;

        // Return error if no prompt found
        if prompt.is_empty() {
            warn!(target: "BaseConnection::find_prompt", "Unable to find prompt in output: {:?}", prompt);
            return Err(NetsshError::PromptError(
                "Unable to find prompt".to_string(),
            ));
        }

        debug!(target: "BaseConnection::find_prompt", "Found prompt: {:?}", prompt);
        Ok(prompt)
    }

    /// Disable paging on the device by sending a command.
    ///
    /// # Arguments
    ///
    /// * `command` - Device command to disable pagination of output (default: "terminal length 0")
    /// * `cmd_verify` - Verify command echo before proceeding
    /// * `pattern` - Pattern to terminate reading of channel
    ///
    /// # Returns
    ///
    /// Output from the command
    pub fn disable_paging(
        &mut self,
        command: Option<&str>,
        cmd_verify: Option<bool>,
        pattern: Option<&str>,
    ) -> Result<String, NetsshError> {
        let command = command.unwrap_or("terminal length 0");
        let cmd_verify = cmd_verify.unwrap_or(true);

        debug!(target: "BaseConnection::disable_paging", "Disabling paging with command: {}", command);

        // Normalize command and write to channel
        let cmd = format!("{}\n", command.trim());
        self.write_channel(&cmd)?;

        // Read until we detect the command echo (avoid getting out of sync)
        let mut output = String::new();
        if cmd_verify {
            debug!(target: "BaseConnection::disable_paging", "Verifying command echo");
            match self.read_until_pattern(&regex::escape(command.trim()), Some(20.0), None) {
                Ok(data) => {
                    output.push_str(&data);
                }
                Err(e) => {
                    warn!(target: "BaseConnection::disable_paging", "Command echo verification failed: {}", e);
                    // Continue despite failure
                }
            }
        }

        // If pattern is provided, read until pattern
        if let Some(pat) = pattern {
            debug!(target: "BaseConnection::disable_paging", "Reading until pattern: {}", pat);
            match self.read_until_pattern(pat, None, None) {
                Ok(data) => {
                    output.push_str(&data);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        } else {
            // Otherwise read until prompt
            debug!(target: "BaseConnection::disable_paging", "Reading until prompt");
            match self.read_until_prompt(None, None, None) {
                Ok(data) => {
                    output.push_str(&data);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        debug!(target: "BaseConnection::disable_paging", "Paging disabled successfully");
        Ok(output)
    }

    /// Prepare the session after the connection has been established.
    ///
    /// This method handles some differences that occur between various devices
    /// early on in the session.
    ///
    /// In general, it should include:
    /// - Testing the channel read
    /// - Setting the base prompt
    /// - Setting terminal width
    /// - Disabling paging
    #[instrument(
        skip_all,
        level = "debug",
        name = "BaseConnection::session_preparation"
    )]
    pub fn session_preparation(&mut self) -> Result<(), NetsshError> {
        debug!("Preparing session");

        // Test the channel read
        self.test_channel_read(None, None)?;

        // Set base prompt with default values
        self.set_base_prompt(None, None, None, None)?;

        // Set terminal width (default implementation does nothing)
        self.set_terminal_width(None, None, None, None)?;

        // Disable paging
        self.disable_paging(None, None, None)?;

        debug!(target: "BaseConnection::session_preparation", "Session preparation complete");
        Ok(())
    }

    /// Test if the channel is responsive by sending a newline and reading response.
    ///
    /// # Arguments
    ///
    /// * `count` - Number of read attempts (default: 40)
    /// * `pattern` - Pattern to search for to determine end of read
    ///
    /// # Returns
    ///
    /// The data read from the channel
    pub fn test_channel_read(
        &mut self,
        count: Option<usize>,
        pattern: Option<&str>,
    ) -> Result<String, NetsshError> {
        let count = count.unwrap_or(40);

        debug!(target: "BaseConnection::test_channel_read", "Testing channel read with count={}", count);

        // If pattern is provided, use read_until_pattern
        if let Some(pat) = pattern {
            debug!(target: "BaseConnection::test_channel_read", "Reading until pattern: {}", pat);
            return self.read_until_pattern(pat, Some(20.0), None);
        }

        // Calculate initial delay
        let mut main_delay = 0.1;

        // Initial sleep
        thread::sleep(Duration::from_secs_f64(main_delay * 10.0));

        let mut new_data = String::new();
        let mut i = 0;

        while i <= count {
            // Try reading from channel
            match self.read_channel_timing(None, Some(20.0)) {
                Ok(data) => {
                    new_data.push_str(&data);
                    if !new_data.is_empty() {
                        debug!(target: "BaseConnection::test_channel_read", "Successfully read data from channel");
                        return Ok(new_data);
                    }
                }
                Err(e) => {
                    if !matches!(e, NetsshError::TimeoutError(_)) {
                        return Err(e);
                    }
                    // On timeout, just continue
                }
            }

            // Write a newline and try again
            debug!(target: "BaseConnection::test_channel_read", "No data received, sending newline (attempt {})", i+1);
            self.write_channel("\n")?;

            // Increment delay using the helper method
            main_delay = Self::increment_delay(main_delay, 1.1, 8.0);
            thread::sleep(Duration::from_secs_f64(main_delay));

            i += 1;
        }

        // If we get here, we've timed out
        Err(NetsshError::TimeoutError(
            "Timed out waiting for data".to_string(),
        ))
    }

    /// Set terminal width to avoid line wrapping issues.
    ///
    /// Default implementation does nothing. Device-specific subclasses should override
    /// this method if the device requires setting terminal width.
    ///
    /// # Arguments
    ///
    /// * `command` - Command to set terminal width
    /// * `cmd_verify` - Whether to verify command echo
    /// * `pattern` - Pattern to terminate reading of channel
    ///
    /// # Returns
    ///
    /// Output from the command if sent, empty string otherwise
    pub fn set_terminal_width(
        &mut self,
        command: Option<&str>,
        cmd_verify: Option<bool>,
        pattern: Option<&str>,
        _delay_factor: Option<f64>, // Kept for API compatibility
    ) -> Result<String, NetsshError> {
        // If no command provided, do nothing
        if command.is_none() || command.unwrap().is_empty() {
            return Ok(String::new());
        }

        let command = command.unwrap();
        let cmd_verify = cmd_verify.unwrap_or(false); // Default to false for terminal width commands

        debug!(target: "BaseConnection::set_terminal_width", "Setting terminal width with command: {}", command);

        // Normalize and write command
        let cmd = format!("{}\n", command.trim());
        self.write_channel(&cmd)?;

        // Get command echo if requested
        let mut output = String::new();
        if cmd_verify {
            match self.read_until_pattern(&regex::escape(command.trim()), None, None) {
                Ok(data) => {
                    output.push_str(&data);
                }
                Err(e) => {
                    warn!(target: "BaseConnection::set_terminal_width", "Command echo verification failed: {}", e);
                    // Continue despite failure
                }
            }
        }

        // If pattern provided, read until pattern
        if let Some(pat) = pattern {
            match self.read_until_pattern(pat, None, None) {
                Ok(data) => {
                    output.push_str(&data);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        } else {
            // Otherwise read until prompt
            match self.read_until_prompt(None, None, None) {
                Ok(data) => {
                    output.push_str(&data);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        debug!(target: "BaseConnection::set_terminal_width", "Terminal width set successfully");
        Ok(output)
    }

    /// Check if the device is in configuration mode.
    ///
    /// # Arguments
    ///
    /// * `check_string` - String that identifies configuration mode
    /// * `pattern` - Pattern to terminate reading of channel
    /// * `force_regex` - Use regex pattern to find check_string in output
    ///
    /// # Returns
    ///
    /// Boolean indicating whether device is in configuration mode
    pub fn check_config_mode(
        &mut self,
        check_string: Option<&str>,
        pattern: Option<&str>,
        force_regex: Option<bool>,
    ) -> Result<bool, NetsshError> {
        let check_string = check_string.unwrap_or("");
        let force_regex = force_regex.unwrap_or(false);

        debug!(target: "BaseConnection::check_config_mode",
            "Checking config mode with string: '{}', pattern: {:?}, force_regex: {}",
            check_string, pattern, force_regex
        );

        // Send return to get prompt
        self.write_channel("\n")?;

        // Read output - prefer delay-based solution if no pattern provided
        let output = if pattern.is_none() {
            self.read_channel_timing(None, Some(10.0))?
        } else {
            self.read_until_pattern(pattern.unwrap(), None, None)?
        };

        debug!(target: "BaseConnection::check_config_mode", "Read output: {}", output);

        // Check if in config mode
        let in_config_mode = if force_regex {
            // Use regex search
            match Regex::new(check_string) {
                Ok(re) => re.is_match(&output),
                Err(e) => {
                    warn!(target: "BaseConnection::check_config_mode", "Invalid regex pattern: {}", e);
                    false
                }
            }
        } else {
            // Simple string contains
            output.contains(check_string)
        };

        debug!(target: "BaseConnection::check_config_mode", "In config mode: {}", in_config_mode);
        Ok(in_config_mode)
    }

    /// Enter configuration mode.
    ///
    /// # Arguments
    ///
    /// * `config_command` - Command to enter configuration mode
    /// * `pattern` - Pattern to terminate reading of channel
    /// * `re_flags` - Regular expression flags (not used in Rust implementation)
    ///
    /// # Returns
    ///
    /// Output from entering configuration mode
    pub fn config_mode(
        &mut self,
        config_command: Option<&str>,
        pattern: Option<&str>,
        _re_flags: Option<i32>,
    ) -> Result<String, NetsshError> {
        let config_command = config_command.unwrap_or("configure terminal");

        debug!(target: "BaseConnection::config_mode", "Entering config mode with command: {}", config_command);

        // Normalize command and write to channel
        let cmd = format!("{}\n", config_command.trim());
        self.write_channel(&cmd)?;

        // Make sure to read until command echo
        let mut output = String::new();
        // match self.read_until_pattern(&regex::escape(config_command.trim()), None, None) {
        //     Ok(data) => {
        //         output.push_str(&data);
        //     }
        //     Err(e) => {
        //         warn!(target: "BaseConnection::config_mode", "Command echo verification failed: {}", e);
        //         // Continue despite failure
        //     }
        // }

        debug!(target: "BaseConnection::config_mode", "Output after command echo verification: bytes {}", output.len());

        // Read until pattern or prompt
        if let Some(pat) = pattern {
            match self.read_until_pattern(pat, None, None) {
                Ok(data) => {
                    output.push_str(&data);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        } else {
            match self.read_until_prompt(Some(10.0), Some(true), None) {
                Ok(data) => {
                    output.push_str(&data);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        debug!(target: "BaseConnection::config_mode", "Output after reading until pattern or prompt: bytes {}", output.len());
        // Verify we're now in config mode
        if !self.check_config_mode(None, None, None)? {
            return Err(NetsshError::ConfigError(
                "Failed to enter configuration mode".to_string(),
            ));
        }

        debug!(target: "BaseConnection::config_mode", "Successfully entered config mode");
        Ok(output)
    }

    /// Exit from configuration mode.
    ///
    /// # Arguments
    ///
    /// * `exit_config` - Command to exit configuration mode
    /// * `pattern` - Pattern to terminate reading of channel
    ///
    /// # Returns
    ///
    /// Output from exiting configuration mode
    pub fn exit_config_mode(
        &mut self,
        exit_config: Option<&str>,
        pattern: Option<&str>,
    ) -> Result<String, NetsshError> {
        let exit_config = exit_config.unwrap_or("end");

        debug!(target: "BaseConnection::exit_config_mode", "Exiting config mode with command: {}, pattern: {}", exit_config, pattern.unwrap_or(""));

        // Check if in config mode
        if !self.check_config_mode(None, None, None)? {
            debug!(target: "BaseConnection::exit_config_mode", "Not in config mode, nothing to do");
            return Ok(String::new());
        }

        // Normalize command and write to channel
        let cmd = format!("{}\n", exit_config.trim());
        self.write_channel(&cmd)?;

        // Make sure to read until command echo
        let mut output = String::new();
        // match self.read_until_pattern(&regex::escape(exit_config.trim()), None, None) {
        //     Ok(data) => {
        //         output.push_str(&data);
        //     }
        //     Err(e) => {
        //         warn!(target: "BaseConnection::exit_config_mode", "Command echo verification failed: {}", e);
        //         // Continue despite failure
        //     }
        // }

        // Read until pattern or prompt
        if let Some(pat) = pattern {
            match self.read_until_pattern(pat, None, None) {
                Ok(data) => {
                    output.push_str(&data);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        } else {
            match self.read_until_prompt(None, Some(true), None) {
                Ok(data) => {
                    output.push_str(&data);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        debug!(target: "BaseConnection::exit_config_mode", "verifying if exited config mode, output: {}", output.len());
        // Verify we're now out of config mode
        if self.check_config_mode(Some(")#"), Some("[>#]"), None)? {
            return Err(NetsshError::ConfigError(
                "Failed to exit configuration mode".to_string(),
            ));
        }

        debug!(target: "BaseConnection::exit_config_mode", "Successfully exited config mode");
        Ok(output)
    }

    /// Check if the SSH connection is alive.
    ///
    /// # Returns
    ///
    /// Boolean indicating whether the connection is alive
    pub fn is_alive(&mut self) -> bool {
        debug!(target: "BaseConnection::is_alive", "Checking if connection is alive");

        // Check if we have a session
        if self.session.is_none() {
            debug!(target: "BaseConnection::is_alive", "Connection is not initialized, is_alive returns false");
            return false;
        }

        // Try sending a null byte to maintain the connection
        match self.write_channel("\0") {
            Ok(_) => {
                // Check SSH connection status
                if let Some(ref session) = self.session {
                    let is_active = session.authenticated();
                    debug!(target: "BaseConnection::is_alive", "Connection is_active: {}", is_active);
                    is_active
                } else {
                    false
                }
            }
            Err(e) => {
                debug!(target: "BaseConnection::is_alive", "Unable to send: {}", e);
                // If unable to send, we can tell for sure that the connection is unusable
                false
            }
        }
    }

    /// Check if in enable mode. Return boolean.
    ///
    /// # Arguments
    ///
    /// * `check_string` - String pattern to identify privilege mode
    ///
    /// # Returns
    ///
    /// Boolean indicating whether the device is in enable mode
    pub fn check_enable_mode(&mut self, check_string: Option<&str>) -> Result<bool, NetsshError> {
        let check_string = check_string.unwrap_or("#");

        debug!(target: "BaseConnection::check_enable_mode", "Checking enable mode with string: {}", check_string);

        // Send newline to get prompt
        self.write_channel("\n")?;

        // Read output including the full line with prompt
        let output = self.read_until_prompt(None, Some(true), None)?;

        debug!(target: "BaseConnection::check_enable_mode", "Read output: {}", output);

        // Check if output contains the enable prompt indicator
        let is_enabled = output.contains(check_string);

        debug!(target: "BaseConnection::check_enable_mode", "In enable mode: {}", is_enabled);
        Ok(is_enabled)
    }

    /// Enter enable mode.
    ///
    /// # Arguments
    ///
    /// * `cmd` - Command to enter enable mode
    /// * `pattern` - Pattern to search for indicating device is waiting for password
    /// * `enable_pattern` - Pattern indicating you have entered enable mode
    /// * `check_state` - Whether to check if already in enable mode before proceeding
    /// * `re_flags` - Regular expression flags (not used in Rust implementation)
    ///
    /// # Returns
    ///
    /// Output from entering enable mode
    pub fn enable(
        &mut self,
        cmd: Option<&str>,
        pattern: Option<&str>,
        enable_pattern: Option<&str>,
        check_state: Option<bool>,
        _re_flags: Option<i32>, // Not used in Rust implementation
    ) -> Result<String, NetsshError> {
        let cmd = cmd.unwrap_or("enable");
        let pattern = pattern.unwrap_or("assword");
        let check_state = check_state.unwrap_or(true);

        let mut output = String::new();

        debug!(target: "BaseConnection::enable", "Entering enable mode with command: {}", cmd);

        // Check if already in enable mode and skip if so
        if check_state && self.check_enable_mode(None)? {
            debug!(target: "BaseConnection::enable", "Already in enable mode");
            return Ok(output);
        }

        // Send enable command
        let cmd_str = format!("{}\n", cmd.trim());
        self.write_channel(&cmd_str)?;

        // Read command echo
        if let Ok(data) = self.read_until_pattern(&regex::escape(cmd.trim()), None, None) {
            output.push_str(&data);
        }

        // Read until prompt or password pattern
        let prompt_pattern = if let Some(base_prompt) = &self.base_prompt {
            regex::escape(base_prompt)
        } else {
            String::new()
        };

        let combined_pattern = if !prompt_pattern.is_empty() {
            format!(r"(?:{}|{})", pattern, prompt_pattern)
        } else {
            pattern.to_string()
        };

        match self.read_until_pattern(&combined_pattern, None, None) {
            Ok(data) => {
                output.push_str(&data);

                // If password prompt appears, send the secret
                if output.contains(pattern) {
                    debug!(target: "BaseConnection::enable", "Password prompt detected, sending secret");

                    // Check if secret is available in config
                    if let Some(secret) = &self.config.secret {
                        // Send the secret
                        let secret_str = format!("{}\n", secret);
                        self.write_channel(&secret_str)?;

                        // Read the response
                        match self.read_until_prompt(None, None, None) {
                            Ok(data) => {
                                output.push_str(&data);
                            }
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    } else {
                        return Err(NetsshError::AuthenticationError(
                            "Enable password required but not provided".to_string(),
                        ));
                    }
                }

                // Check for enable pattern if provided
                if let Some(enable_pat) = enable_pattern {
                    if !output.contains(enable_pat) {
                        match self.read_until_pattern(enable_pat, None, None) {
                            Ok(data) => {
                                output.push_str(&data);
                            }
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    }
                }

                // Verify we're in enable mode
                if !self.check_enable_mode(None)? {
                    return Err(NetsshError::AuthenticationError(
                        "Failed to enter enable mode".to_string(),
                    ));
                }
            }
            Err(e) => {
                return Err(e);
            }
        }

        debug!(target: "BaseConnection::enable", "Successfully entered enable mode");
        Ok(output)
    }

    /// Exit enable mode.
    ///
    /// # Arguments
    ///
    /// * `exit_command` - Command to exit enable mode
    ///
    /// # Returns
    ///
    /// Output from exiting enable mode
    pub fn exit_enable_mode(&mut self, exit_command: Option<&str>) -> Result<String, NetsshError> {
        let exit_command = exit_command.unwrap_or("disable");

        debug!(target: "BaseConnection::exit_enable_mode", "Exiting enable mode with command: {}", exit_command);

        // Check if actually in enable mode
        if !self.check_enable_mode(None)? {
            debug!(target: "BaseConnection::exit_enable_mode", "Not in enable mode, nothing to do");
            return Ok(String::new());
        }

        // Send exit command
        let cmd = format!("{}\n", exit_command.trim());
        self.write_channel(&cmd)?;

        // Read response
        let output = self.read_until_prompt(None, None, None)?;

        // Verify we're out of enable mode
        if self.check_enable_mode(None)? {
            return Err(NetsshError::OperationError(
                "Failed to exit enable mode".to_string(),
            ));
        }

        debug!(target: "BaseConnection::exit_enable_mode", "Successfully exited enable mode");
        Ok(output)
    }

    /// Cleanup before disconnecting. Default implementation sends 'exit' to device.
    ///
    /// Device-specific classes can override this method to provide alternative cleanup steps.
    ///
    /// # Arguments
    ///
    /// * `command` - Command to send to logout of the device
    ///
    /// # Returns
    ///
    /// Result indicating success or error
    pub fn cleanup(&mut self, command: Option<&str>) -> Result<(), NetsshError> {
        let logout_command = command.unwrap_or("exit");

        debug!(target: "BaseConnection::cleanup", "Cleaning up session with command: {}", logout_command);

        // Send logout command
        self.write_channel(&format!("{}\n", logout_command))?;

        // Short wait to ensure command is processed
        thread::sleep(Duration::from_millis(500));

        Ok(())
    }

    /// Try to gracefully close the connection.
    ///
    /// First calls cleanup_session() to ensure we're properly exiting all device modes,
    /// then closes the SSH connection.
    ///
    /// # Returns
    ///
    /// Result indicating success or error
    pub fn disconnect(&mut self) -> Result<(), NetsshError> {
        debug!(target: "BaseConnection::disconnect", "Disconnecting from device");

        // Try to gracefully exit all modes and send exit command
        let _ = self.cleanup_session(None, None);

        // Close SSH connection
        if let Some(session) = self.session.take() {
            debug!(target: "BaseConnection::disconnect", "Closing SSH session");

            // Close the session
            if let Err(e) = session.disconnect(None, "Closing connection", None) {
                warn!(target: "BaseConnection::disconnect", "Error during disconnect: {}", e);
                // Continue despite errors
            }
        }

        // Close the session log
        if self.session_log.is_enabled() {
            debug!(target: "BaseConnection::disconnect", "Closing session log");
            self.session_log.disable();
        }

        debug!(target: "BaseConnection::disconnect", "Disconnection complete");
        Ok(())
    }

    /// Close the SSH connection without cleanup.
    ///
    /// Use disconnect() for graceful disconnection with cleanup.
    pub fn close(&mut self) -> Result<(), NetsshError> {
        if let Some(session) = self.session.take() {
            if let Err(e) = session.disconnect(None, "Closing connection", None) {
                warn!(target: "BaseConnection::close", "Error during close: {}", e);
                // Continue despite errors
            }
        }

        // Close the session log
        if self.session_log.is_enabled() {
            self.session_log.disable();
        }

        Ok(())
    }

    /// Simple wrapper for send_command with default parameters
    ///
    /// # Arguments
    ///
    /// * `command_string` - Command to send to the device
    ///
    /// # Returns
    ///
    /// Output from the command
    // #[instrument(
    //     skip_all,
    //     level = "debug",
    //     name = "BaseConnection::send_command_simple"
    // )]
    pub fn send_command_simple(&mut self, command_string: &str) -> Result<String, NetsshError> {
        debug!(target: "BaseConnection::send_command_simple", "Sending command: {}", command_string);

        // Call the full method with default parameters
        self.send_command(
            command_string,
            None, // expect_string
            None, // read_timeout
            None, // auto_find_prompt
            None, // strip_prompt
            None, // strip_command
            None, // normalize
            None, // cmd_verify
        )
    }

    /// Thoroughly cleanup the session before disconnecting.
    ///
    /// This method ensures we exit configuration mode and privilege mode (if in them)
    /// before disconnecting, to leave the device in a clean state.
    ///
    /// # Arguments
    ///
    /// * `exit_config_command` - Command to exit config mode (if needed)
    /// * `exit_enable_command` - Command to exit enable mode (if needed)
    ///
    /// # Returns
    ///
    /// Result indicating success or error
    pub fn cleanup_session(
        &mut self,
        exit_config_command: Option<&str>,
        exit_enable_command: Option<&str>,
    ) -> Result<(), NetsshError> {
        debug!(target: "BaseConnection::cleanup_session", "Performing thorough session cleanup");

        // First check if we're in config mode and exit if needed
        match self.check_config_mode(None, None, None) {
            Ok(in_config) => {
                if in_config {
                    debug!(target: "BaseConnection::cleanup_session", "Exiting configuration mode");
                    if let Err(e) = self.exit_config_mode(exit_config_command, None) {
                        warn!(target: "BaseConnection::cleanup_session", "Error exiting config mode: {}", e);
                        // Continue despite error
                    }
                }
            }
            Err(e) => {
                warn!(target: "BaseConnection::cleanup_session", "Error checking config mode: {}", e);
                // Continue despite error
            }
        }

        // Then check if we're in enable mode and exit if needed
        match self.check_enable_mode(None) {
            Ok(in_enable) => {
                if in_enable {
                    debug!(target: "BaseConnection::cleanup_session", "Exiting enable mode");
                    if let Err(e) = self.exit_enable_mode(exit_enable_command) {
                        warn!(target: "BaseConnection::cleanup_session", "Error exiting enable mode: {}", e);
                        // Continue despite error
                    }
                }
            }
            Err(e) => {
                warn!(target: "BaseConnection::cleanup_session", "Error checking enable mode: {}", e);
                // Continue despite error
            }
        }

        // Finish with standard cleanup (send exit command)
        self.cleanup(None)?;

        debug!(target: "BaseConnection::cleanup_session", "Session cleanup complete");
        Ok(())
    }

    /// Read until either self.base_prompt or pattern is detected.
    ///
    /// # Arguments
    ///
    /// * `pattern` - Pattern to terminate reading of channel
    /// * `read_timeout` - Maximum time to wait looking for prompt/pattern
    /// * `read_entire_line` - If true, reads the entire line containing the prompt
    /// * `re_flags` - Regex flags (not used in Rust implementation)
    ///
    /// # Returns
    ///
    /// The accumulated output up to and including the prompt or pattern
    pub fn read_until_prompt_or_pattern(
        &mut self,
        pattern: Option<&str>,
        read_timeout: Option<f64>,
        read_entire_line: Option<bool>,
        re_flags: Option<i32>,
    ) -> Result<String, NetsshError> {
        debug!(target: "BaseConnection::read_until_prompt_or_pattern", "Reading until prompt or pattern");

        // Get the base prompt or return error if not set
        let base_prompt = self
            .base_prompt
            .as_ref()
            .ok_or_else(|| NetsshError::ConnectionError("Base prompt not set".to_string()))?;

        // Escape the prompt for regex and construct pattern
        let prompt_pattern = if read_entire_line.unwrap_or(false) {
            format!("{}.*", regex::escape(base_prompt))
        } else {
            regex::escape(base_prompt)
        };

        // If pattern is provided, combine it with the prompt pattern
        let combined_pattern = if let Some(pat) = pattern {
            // Use non-capturing group to combine patterns
            format!(r"(?:{}|{})", prompt_pattern, pat)
        } else {
            prompt_pattern
        };

        debug!(target: "BaseConnection::read_until_prompt_or_pattern",
            "Reading with combined pattern: {}, timeout: {:?}",
            combined_pattern, read_timeout
        );

        // Delegate to read_until_pattern with the combined pattern
        self.read_until_pattern(&combined_pattern, read_timeout, re_flags)
    }

    /// Strip any backspace characters out of the output.
    ///
    /// # Arguments
    ///
    /// * `output` - Output obtained from a remote network device.
    ///
    /// # Returns
    ///
    /// String with backspace characters removed
    pub fn strip_backspaces(&self, output: &str) -> String {
        // ASCII backspace character
        let backspace_char = "\x08";
        output.replace(backspace_char, "")
    }

    /// Send multiple commands, each with a potentially different expect_string.
    ///
    /// # Arguments
    ///
    /// * `commands` - A vector of either:
    ///                 1. [cmd1, cmd2, cmd3, ...] where default expect string (prompt) is used
    ///                 2. [[cmd1, expect1], [cmd2, expect2], ...] where expect string is specified
    /// * `multiline` - Whether to treat this as a multiline operation (changes parameter handling)
    /// * `read_timeout` - Timeout for reading responses from the device
    /// * `auto_find_prompt` - Whether to automatically find prompt
    /// * `strip_prompt` - Whether to strip trailing prompt from output
    /// * `strip_command` - Whether to strip command echo from output
    /// * `normalize` - Whether to normalize command newlines
    /// * `cmd_verify` - Whether to verify command echoes
    ///
    /// # Returns
    ///
    /// The combined output from all commands
    pub fn send_multiline(
        &mut self,
        commands: &[Vec<&str>],
        multiline: Option<bool>,
        read_timeout: Option<f64>,
        auto_find_prompt: Option<bool>,
        strip_prompt: Option<bool>,
        strip_command: Option<bool>,
        normalize: Option<bool>,
        cmd_verify: Option<bool>,
    ) -> Result<String, NetsshError> {
        debug!(target: "BaseConnection::send_multiline", "Sending {} multiline commands", commands.len());

        // Configure multiline parameters
        let multiline = multiline.unwrap_or(true);
        let strip_prompt = if multiline { Some(false) } else { strip_prompt };
        let strip_command = if multiline {
            Some(false)
        } else {
            strip_command
        };

        // Initialize output string
        let mut output = String::new();

        // Get default expect string
        let default_expect_string = if let Some(prompt) = &self.base_prompt {
            regex::escape(prompt)
        } else {
            // Try to find prompt if not set
            let prompt = self.find_prompt(None, None)?;
            regex::escape(&prompt)
        };

        // Process commands based on format
        for command_item in commands {
            if command_item.len() == 1 {
                // Single item - use default expect string
                let cmd = command_item[0];
                let command_output = self.send_command(
                    cmd,
                    Some(&default_expect_string),
                    read_timeout,
                    auto_find_prompt,
                    strip_prompt,
                    strip_command,
                    normalize,
                    cmd_verify,
                )?;
                output.push_str(&command_output);
            } else if command_item.len() >= 2 {
                // Two items - cmd and expect string
                let cmd = command_item[0];
                let expect_string = if command_item[1].is_empty() {
                    &default_expect_string
                } else {
                    command_item[1]
                };

                let command_output = self.send_command(
                    cmd,
                    Some(expect_string),
                    read_timeout,
                    auto_find_prompt,
                    strip_prompt,
                    strip_command,
                    normalize,
                    cmd_verify,
                )?;
                output.push_str(&command_output);
            }
        }

        debug!(target: "BaseConnection::send_multiline", "Multiline commands complete");
        Ok(output)
    }

    /// Send multiple commands in timing mode (delay-based, not pattern-based).
    ///
    /// # Arguments
    ///
    /// * `commands` - A vector of command strings to send
    /// * `multiline` - Whether to treat this as a multiline operation (changes parameter handling)
    /// * `last_read` - Amount of time to wait before performing one last read
    /// * `read_timeout` - Absolute timer for how long to keep reading data
    /// * `strip_prompt` - Whether to strip trailing prompt from output
    /// * `strip_command` - Whether to strip command echo from output
    /// * `normalize` - Whether to normalize command newlines
    ///
    /// # Returns
    ///
    /// The combined output from all commands
    pub fn send_multiline_timing(
        &mut self,
        commands: &[&str],
        multiline: Option<bool>,
        last_read: Option<f64>,
        read_timeout: Option<f64>,
        strip_prompt: Option<bool>,
        strip_command: Option<bool>,
        normalize: Option<bool>,
    ) -> Result<String, NetsshError> {
        debug!(target: "BaseConnection::send_multiline_timing", "Sending {} multiline commands in timing mode", commands.len());

        // Configure multiline parameters
        let multiline = multiline.unwrap_or(true);
        let strip_prompt = if multiline { Some(false) } else { strip_prompt };
        let strip_command = if multiline {
            Some(false)
        } else {
            strip_command
        };

        // Initialize output string
        let mut output = String::new();

        // Send each command
        for cmd in commands {
            debug!(target: "BaseConnection::send_multiline_timing", "Sending command: {}", cmd);

            // Normalize command if needed
            let normalized_cmd = if normalize.unwrap_or(true) {
                format!("{}\n", cmd.trim())
            } else {
                cmd.to_string()
            };

            // Write the command
            self.write_channel(&normalized_cmd)?;

            // Read response using timing-based approach
            let command_output = self.read_channel_timing(last_read, read_timeout)?;

            // Process output
            let processed_output = if strip_command.unwrap_or(false) {
                self.strip_command(cmd, &command_output)
            } else {
                command_output
            };

            let final_output = if strip_prompt.unwrap_or(false) {
                self.strip_prompt(&processed_output)
            } else {
                processed_output
            };

            output.push_str(&final_output);
        }

        debug!(target: "BaseConnection::send_multiline_timing", "Multiline timing commands complete");
        Ok(output)
    }

    /// Save the running configuration to the startup configuration.
    ///
    /// This method should be overridden by device-specific subclasses
    /// as the implementation varies by platform.
    ///
    /// # Arguments
    ///
    /// * `cmd` - Command to save the configuration
    /// * `confirm` - Whether the save requires confirmation
    /// * `confirm_response` - Response to send if confirmation is required
    ///
    /// # Returns
    ///
    /// The output from the save command
    pub fn save_config(
        &mut self,
        _cmd: Option<&str>,
        _confirm: Option<bool>,
        _confirm_response: Option<&str>,
    ) -> Result<String, NetsshError> {
        debug!(target: "BaseConnection::save_config", "Base implementation of save_config does nothing");
        Err(NetsshError::UnsupportedOperation(
            "Network device does not support 'save_config()' method".to_string(),
        ))
    }

    /// Commit the candidate configuration (for platforms that support commit).
    ///
    /// This method should be overridden by device-specific subclasses
    /// as the implementation varies by platform.
    ///
    /// # Returns
    ///
    /// The output from the commit command
    pub fn commit(&mut self) -> Result<String, NetsshError> {
        debug!(target: "BaseConnection::commit", "Base implementation of commit does nothing");
        Err(NetsshError::UnsupportedOperation(
            "Network device does not support 'commit()' method".to_string(),
        ))
    }

    /// Execute command_string on the SSH channel using a delay-based mechanism.
    ///
    /// Generally used for show commands. The method will keep reading data until there
    /// is no more data to be read or a timeout occurs.
    ///
    /// # Arguments
    ///
    /// * `command_string` - The command to execute on the network device
    /// * `last_read` - Duration to wait for last read (default: 2.0 seconds)
    /// * `read_timeout` - Maximum time to wait for data (default: 120.0 seconds)
    /// * `strip_prompt` - Remove the trailing prompt from the output
    /// * `strip_command` - Remove the command from the output
    /// * `normalize` - Normalize the command before sending
    /// * `cmd_verify` - Whether to verify command echo before proceeding
    ///
    /// # Returns
    ///
    /// The output from the command
    pub fn send_command_timing(
        &mut self,
        command_string: &str,
        last_read: Option<f64>,
        read_timeout: Option<f64>,
        strip_prompt: Option<bool>,
        strip_command: Option<bool>,
        normalize: Option<bool>,
        cmd_verify: Option<bool>,
    ) -> Result<String, NetsshError> {
        debug!(target: "BaseConnection::send_command_timing", "Sending command: {}", command_string);

        // Default values
        let strip_prompt = strip_prompt.unwrap_or(true);
        let strip_command = strip_command.unwrap_or(true);
        let normalize = normalize.unwrap_or(true);
        let cmd_verify = cmd_verify.unwrap_or(false);

        let mut output = String::new();

        // Normalize command if needed
        let cmd = if normalize {
            format!("{}\n", command_string.trim())
        } else {
            command_string.to_string()
        };

        // Write command to channel
        self.write_channel(&cmd)?;

        // Command verification - make sure we see our command echoed back
        if cmd_verify {
            let cmd_trim = command_string.trim();
            if !cmd_trim.is_empty() {
                debug!(target: "BaseConnection::send_command_timing", "Verifying command echo: {}", cmd_trim);
                match self.read_until_pattern(&regex::escape(cmd_trim), Some(10.0), None) {
                    Ok(echo_data) => {
                        output.push_str(&echo_data);
                    }
                    Err(e) => {
                        debug!(target: "BaseConnection::send_command_timing", "Command echo verification failed: {}", e);
                        // Continue anyway, but note the failure
                    }
                }
            }
        }

        // Read the response using timing-based approach
        let timing_output = self.read_channel_timing(last_read, read_timeout)?;
        output.push_str(&timing_output);

        // Sanitize output if requested
        let sanitized_output =
            self._sanitize_output(&output, strip_command, Some(command_string), strip_prompt);

        debug!(target: "BaseConnection::send_command_timing", "Command complete, response length: {}", sanitized_output.len());
        Ok(sanitized_output)
    }

    // Helper method to handle first line with potential backspace characters
    fn first_line_handler(&self, data: &str, search_pattern: &str) -> (String, bool) {
        // Try to process the first line to handle backspace characters
        let lines: Vec<&str> = data.split("\n").collect();
        if lines.is_empty() {
            return (data.to_string(), false);
        }

        let first_line = lines[0];
        if first_line.contains("\u{0008}") {
            // Backspace character
            // Pattern to find and remove the search pattern and anything after it on the first line
            let pattern = format!("{}.*$", regex::escape(search_pattern));
            if let Ok(re) = Regex::new(&pattern) {
                let first_line_replaced = re.replace(first_line, "").to_string();
                let mut new_lines = lines.clone();
                new_lines[0] = &first_line_replaced;
                return (new_lines.join("\n"), true);
            }
        }

        (data.to_string(), true)
    }

    /// Send configuration commands using a more flexible approach, with extensive options.
    ///
    /// # Arguments
    ///
    /// * `config_commands` - Iterator of configuration commands to send to the device
    /// * `exit_config_mode` - Whether to exit config mode after completion
    /// * `read_timeout` - Timeout for reading responses from the device
    /// * `strip_prompt` - Whether to strip trailing prompt from output
    /// * `strip_command` - Whether to strip command echo from output
    /// * `config_mode_command` - Command to enter config mode
    /// * `cmd_verify` - Whether to verify command echoes
    /// * `enter_config_mode` - Whether to enter config mode before sending commands
    /// * `error_pattern` - Regex pattern to detect configuration errors
    /// * `terminator` - Alternate terminator pattern
    /// * `bypass_commands` - Regex pattern for commands that should bypass verification
    /// * `fast_cli` - Whether to use fast mode (minimal verification)
    ///
    /// # Returns
    ///
    /// The combined output from all commands
    pub fn send_config_set(
        &mut self,
        config_commands: Vec<String>,
        exit_config_mode: Option<bool>,
        read_timeout: Option<f64>,
        strip_prompt: Option<bool>,
        strip_command: Option<bool>,
        config_mode_command: Option<&str>,
        cmd_verify: Option<bool>,
        enter_config_mode: Option<bool>,
        error_pattern: Option<&str>,
        terminator: Option<&str>,
        bypass_commands: Option<&str>,
        fast_cli: Option<bool>,
    ) -> Result<String, NetsshError> {
        debug!(target: "BaseConnection::send_config_set_extended", "Sending configuration commands");

        // Default values
        let exit_config_mode = exit_config_mode.unwrap_or(true);
        let read_timeout = read_timeout.unwrap_or(15.0);
        let strip_prompt = strip_prompt.unwrap_or(false);
        let strip_command = strip_command.unwrap_or(false);
        let cmd_verify = cmd_verify.unwrap_or(true);
        let enter_config_mode = enter_config_mode.unwrap_or(true);
        let _terminator = terminator.unwrap_or(r"#");
        let fast_cli = fast_cli.unwrap_or(false);

        if config_commands.is_empty() {
            return Ok(String::new());
        }

        // Bypass pattern for commands where verification should be disabled
        let bypass_commands_pattern = if let Some(pattern) = bypass_commands {
            match Regex::new(pattern) {
                Ok(re) => Some(re),
                Err(e) => {
                    warn!(target: "BaseConnection::send_config_set_extended", "Invalid bypass_commands regex: {}", e);
                    None
                }
            }
        } else {
            // Default bypass for banner commands
            match Regex::new(r"^banner .*$") {
                Ok(re) => Some(re),
                Err(_) => None,
            }
        };

        // Error pattern for detecting configuration errors
        let error_regex = if let Some(pattern) = error_pattern {
            match Regex::new(pattern) {
                Ok(re) => Some(re),
                Err(e) => {
                    warn!(target: "BaseConnection::send_config_set_extended", "Invalid error_pattern regex: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Check for bypass_detected
        let bypass_detected = bypass_commands_pattern.as_ref().map_or(false, |re| {
            config_commands.iter().any(|cmd| re.is_match(cmd))
        });

        // If bypass detected, disable cmd_verify
        let cmd_verify = if bypass_detected {
            debug!(target: "BaseConnection::send_config_set_extended", "Bypass command detected, disabling cmd_verify");
            false
        } else {
            cmd_verify
        };

        // Send config commands
        let mut output = String::new();

        // Enter config mode if requested
        if enter_config_mode {
            if let Some(config_cmd) = config_mode_command {
                let config_output = self.config_mode(Some(config_cmd), None, None)?;
                output.push_str(&config_output);
            } else {
                let config_output = self.config_mode(None, None, None)?;
                output.push_str(&config_output);
            }
        }

        // Fast mode (send all commands at once, minimal verification)
        if fast_cli && !error_regex.is_some() {
            for cmd in &config_commands {
                let cmd_str = format!("{}\n", cmd.trim());
                self.write_channel(&cmd_str)?;
            }

            // Read all output at once
            let cmd_output = self.read_channel_timing(None, Some(read_timeout))?;
            output.push_str(&cmd_output);
        }
        // No command verification mode
        else if !cmd_verify {
            for cmd in &config_commands {
                debug!(target: "BaseConnection::send_config_set_extended", "Sending command without verification: {}", cmd);
                let cmd_str = format!("{}\n", cmd.trim());
                self.write_channel(&cmd_str)?;

                // Short sleep between commands
                thread::sleep(Duration::from_millis(50));

                // If error pattern is specified, check output after each command
                if let Some(re) = &error_regex {
                    let cmd_output = self.read_channel_timing(None, Some(read_timeout))?;
                    output.push_str(&cmd_output);

                    // Check for errors
                    if re.is_match(&output) {
                        let msg = format!("Invalid input detected at command: {}", cmd);
                        return Err(NetsshError::ConfigError(msg));
                    }
                }
            }

            // If no error pattern specified, read all output at once
            if error_regex.is_none() {
                let cmd_output = self.read_channel_timing(None, Some(read_timeout))?;
                output.push_str(&cmd_output);
            }
        }
        // Command verification mode (default)
        else {
            for cmd in &config_commands {
                debug!(target: "BaseConnection::send_config_set_extended", "Sending command with verification: {}", cmd);
                let cmd_str = format!("{}\n", cmd.trim());
                self.write_channel(&cmd_str)?;

                // Make sure command is echoed
                match self.read_until_pattern(&regex::escape(cmd.trim()), Some(read_timeout), None)
                {
                    Ok(data) => {
                        output.push_str(&data);
                    }
                    Err(e) => {
                        warn!(target: "BaseConnection::send_config_set_extended", "Command echo verification failed: {}", e);
                        // Continue despite failure
                    }
                }

                // Read until prompt or terminator pattern
                // let pattern = format!(
                //     "(?:{}.*$|{}.*$)",
                //     self.base_prompt.as_ref().map_or("", |p| p),
                //     terminator
                // );

                // match self.read_until_pattern(&pattern, Some(read_timeout), Some(8)) {
                //     // re.M = 8 in Python
                //     Ok(data) => {
                //         output.push_str(&data);
                //     }
                //     Err(e) => {
                //         return Err(e);
                //     }
                // }

                // Check for errors if error_pattern specified
                if let Some(re) = &error_regex {
                    if re.is_match(&output) {
                        let msg = format!("Invalid input detected at command: {}", cmd);
                        return Err(NetsshError::ConfigError(msg));
                    }
                }
            }
        }

        // Exit config mode if requested
        if exit_config_mode {
            let exit_output = self.exit_config_mode(None, None)?;
            output.push_str(&exit_output);
        }

        // Sanitize output
        let sanitized_output = self._sanitize_output(&output, strip_command, None, strip_prompt);

        debug!(target: "BaseConnection::send_config_set_extended", "Configuration commands sent successfully");
        Ok(sanitized_output)
    }

    /// Set the device type for this connection
    pub fn set_device_type(&mut self, device_type: DeviceType) {
        debug!("Setting device type to {:?}", device_type);
        self.device_type = device_type;
    }
}
