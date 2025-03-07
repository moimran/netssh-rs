use crate::channel::SSHChannel;
use crate::config::NetsshConfig;
use crate::error::NetsshError;
use crate::session_log::SessionLog;
use log::{debug, info};
use regex::Regex;
use ssh2::Session;
use std::net::TcpStream;
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
        let tcp = TcpStream::connect(&addr).map_err(|e| {
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
        let mut session = Session::new().map_err(|e| {
            info!("Failed to create SSH session: {}", e);
            NetsshError::SshError(e)
        })?;
        session.set_tcp_stream(tcp);

        session.set_blocking(true);

        debug!(target: "BaseConnection::connect", "SSH session created, starting handshake");

        debug!(target: "BaseConnection::connect", "Starting SSH handshake");
        session.handshake().map_err(|e| {
            info!("SSH handshake failed: {}", e);
            NetsshError::SshError(e)
        })?;

        debug!(target: "BaseConnection::connect", "SSH handshake completed successfully");

        debug!(target: "BaseConnection::connect", "Authenticating with username {}", username);
        if let Some(pass) = password {
            debug!(target: "BaseConnection::connect", "Attempting password authentication for user: {}", username);
            session.userauth_password(username, pass).map_err(|e| {
                info!("Password authentication failed: {}", e);
                NetsshError::AuthenticationError(e.to_string())
            })?;
        } else {
            debug!(target: "BaseConnection::connect", "Attempting SSH agent authentication for user: {}", username);
            session.userauth_agent(username).map_err(|e| {
                info!("SSH agent authentication failed: {}", e);
                NetsshError::AuthenticationError(e.to_string())
            })?;
        }

        debug!(target: "BaseConnection::connect", "Authentication successful");

        debug!(target: "BaseConnection::connect", "Opening SSH channel");
        let mut channel = session.channel_session().map_err(|e| {
            info!("Failed to create channel session: {}", e);
            NetsshError::SshError(e)
        })?;

        debug!(target: "BaseConnection::connect", "SSH channel created successfully");

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

        self.session = Some(session);
        self.channel = SSHChannel::new(Some(channel));

        info!("Successfully connected to {}:{}", host, port);
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
        data.replace("\r\n", "\n")
    }

    // Helper method to strip ANSI escape codes
    fn strip_ansi_escape_codes(&self, data: &str) -> String {
        // Simple regex to strip ANSI escape codes
        let re = Regex::new(r"\x1B\[[0-9;]*[a-zA-Z]").unwrap();
        re.replace_all(data, "").to_string()
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

        // Use read_timeout_override if set
        let read_timeout = self
            .read_timeout_override
            .unwrap_or(self.config.pattern_timeout);

        // Special case for prompt patterns
        if pattern == "[>#]" && self.base_prompt.is_some() {
            debug!(target: "BaseConnection::read_until_pattern", "Using special handling for prompt pattern");
            return self.read_until_prompt();
        }

        // Compile the regex pattern
        let regex = Regex::new(pattern).map_err(|e| NetsshError::PatternError(e.to_string()))?;

        // If we have a base_prompt and the pattern is for a prompt, use the SSHChannel's read_until_prompt method
        if (pattern.ends_with("#") || pattern.ends_with(">") || pattern.contains("[>#]"))
            && self.base_prompt.is_some()
        {
            debug!(target: "BaseConnection::read_until_pattern", "Using SSHChannel's read_until_prompt method");
            return self
                .channel
                .read_until_prompt(Some(read_timeout), Some(&regex));
        }

        if pattern.ends_with("#") || pattern.ends_with(">") || pattern.contains("[>#]") {
            debug!(target: "BaseConnection::read_until_pattern", "Using SSHChannel's read_until_prompt method with no base prompt");
            return self
                .channel
                .read_until_prompt(Some(read_timeout), Some(&regex));
        }

        let mut output = String::new();
        let loop_delay = Duration::from_millis(10); // 10ms delay between reads
        let start_time = SystemTime::now();

        // Keep reading until timeout or pattern is found
        while start_time.elapsed()? < read_timeout {


            // Read a chunk of data
            std::thread::sleep(Duration::from_millis(500));
            let new_data = self.read_channel()?;

            if !new_data.is_empty() {
                debug!(target: "BaseConnection::read_until_pattern", "Read chunk: {:?}", new_data);
                output.push_str(&new_data);
                debug!(target: "BaseConnection::read_until_pattern", "============== {}",output);

                // Check if pattern is found
                if regex.is_match(&output) {
                    debug!(target: "BaseConnection::read_until_pattern", "Pattern found: {}", pattern);

                    // Process the output to split at the pattern
                    if pattern.contains('(') && !pattern.contains("(?:") {
                        debug!(target: "BaseConnection::read_until_pattern", "Parenthesis found in pattern, may need special handling");
                    }

                    let results = regex.splitn(&output, 2).collect::<Vec<&str>>();

                    if results.len() > 1 && !results[1].is_empty() {
                        // Store excess data in the read buffer
                        self._read_buffer = results[1].to_string();

                        // Return everything up to and including the pattern
                        let match_index = output.len() - results[1].len();
                        return Ok(output[..match_index].to_string());
                    }

                    return Ok(output);
                }
            } else {
                // If no new data, sleep a bit to avoid CPU spinning
                std::thread::sleep(loop_delay);
            }
        }

        // If we have some output but timed out, return what we have instead of an error
        if !output.is_empty() {
            debug!(target: "BaseConnection::read_until_pattern", "Timeout reached but returning partial output: {:?}", output);
            return Ok(output);
        }

        debug!(target: "BaseConnection::read_until_pattern", "Timeout reached with no output");
        Err(NetsshError::TimeoutError(format!(
            "Pattern '{}' not found within timeout period. Final buffer: {:?}",
            pattern, output
        )))
    }

    // Special method to handle reading until a prompt (> or #)
    pub fn read_until_prompt(&mut self) -> Result<String, NetsshError> {
        debug!(target: "BaseConnection::read_until_prompt", "Reading until prompt (> or #)");

        let mut output = String::new();
        let loop_delay = Duration::from_millis(100); // 10ms delay between reads
        let start_time = SystemTime::now();

        // Keep reading until timeout or prompt is found
        while start_time.elapsed()? < self.config.pattern_timeout {
            std::thread::sleep(loop_delay);
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

    pub fn send_command(&mut self, command: &str) -> Result<String, NetsshError> {
        debug!(target: "BaseConnection::send_command", "Sending command: {}", command);

        if self.config.auto_clear_buffer {
            // Clear any pending data in the buffer
            let _ = self.read_channel();
        }

        // Write the command
        self.write_channel(&format!("{}\n", command))?;

        // Wait for command echo and prompt
        // Use a pattern that matches both user mode (>) and privileged mode (#) prompts
        let output = self.read_until_pattern("[>#]")?;

        // Remove command echo from output
        let lines: Vec<&str> = output.lines().collect();
        let result = if lines.len() > 1 {
            // Skip the first line (command echo) and join the rest
            lines[1..].join("\n")
        } else {
            output
        };

        debug!(target: "BaseConnection::send_command", "Command output received, length: {}", result.len());
        Ok(result)
    }

    pub fn set_session_log(&mut self, filename: &str) -> Result<(), NetsshError> {
        self.session_log.enable(filename)
    }
}
