use crate::base_connection::BaseConnection;
use crate::channel::SSHChannel;
use crate::error::NetsshError;
use crate::vendors::cisco::{CiscoBaseConnection, CiscoDeviceConfig};
use async_trait::async_trait;
use regex::Regex;
use lazy_static::lazy_static;
use log::{debug, info, warn};
use std::time::Duration;
use std::thread;

lazy_static! {
    static ref PROMPT_PATTERN: Regex = Regex::new(r"[>#]").unwrap();
    static ref CONFIG_PATTERN: Regex = Regex::new(r"\)#").unwrap();
    static ref PASSWORD_PATTERN: Regex = Regex::new(r"(?i)password").unwrap();
    static ref ANSI_ESCAPE_PATTERN: Regex = Regex::new(r"\x1B\[[0-9;]*[a-zA-Z]").unwrap();
}

pub struct CiscoIosDevice {
    connection: BaseConnection,
    config: CiscoDeviceConfig,
    prompt: String,
    in_enable_mode: bool,
    in_config_mode: bool,
}

impl CiscoIosDevice {
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
        
        info!("Clearing buffer with backoff={}, backoff_max={}, delay_factor={}", 
               backoff, backoff_max, delay_factor);
        
        for _ in 0..10 {
            // Sleep before reading
            let sleep_duration = Duration::from_secs_f64(sleep_time);
            thread::sleep(sleep_duration);
            
            // Read data from channel
            let data = match self.connection.read_channel() {
                Ok(data) => {
                    debug!(target: "CiscoIosDevice::clear_buffer", "Read {} bytes from channel: {:?}", data.len(), data);
                    data
                },
                Err(e) => {
                    warn!(target: "CiscoIosDevice::clear_buffer", "Error reading channel: {}", e);
                    break;
                }
            };
            
            // Strip ANSI escape codes
            let data = self.strip_ansi_escape_codes(&data);
            debug!(target: "CiscoIosDevice::clear_buffer", "After stripping ANSI codes: {:?}", data);
            
            // Add to accumulated output
            output.push_str(&data);
            
            // If no data, we're done
            if data.is_empty() {
                break;
            }
            
            // Double sleep time if backoff is enabled
            debug!(target: "CiscoIosDevice::clear_buffer", "Clear buffer detects data in the channel");
            if backoff {
                sleep_time *= 2.0;
                if sleep_time > backoff_max {
                    sleep_time = backoff_max;
                }
            }
        }
        
        debug!(target: "CiscoIosDevice::clear_buffer", "Buffer cleared, accumulated {} bytes", output.len());
        Ok(output)
    }
    
    /// Strip ANSI escape codes from a string
    fn strip_ansi_escape_codes(&self, data: &str) -> String {
        ANSI_ESCAPE_PATTERN.replace_all(data, "").to_string()
    }

    pub fn close(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoIosDevice::close", "Closing connection to device");
        
        // Try to exit config mode if we're in it
        debug!(target: "CiscoIosDevice::close", "is it in config mode {}", self.in_config_mode);
        if self.in_config_mode {
            let _ = self.exit_config_mode(None);
        }
        
        // Send exit command to gracefully close the connection
        let _ = self.connection.write_channel("exit\n");
        
        // Close the channel if it exists
        if let Some(channel) = self.connection.channel.as_mut() {
            debug!(target: "CiscoIosDevice::close", "Closing SSH channel");
            channel.send_eof().map_err(|e| NetsshError::SshError(e))?;
            channel.wait_eof().map_err(|e| NetsshError::SshError(e))?;
            channel.close().map_err(|e| NetsshError::SshError(e))?;
            channel.wait_close().map_err(|e| NetsshError::SshError(e))?;
        }
        
        // Clear the channel reference
        self.connection.channel = SSHChannel::new(None);
        
        // Clear the session reference (which will drop and close the session)
        self.connection.session = None;
        
        debug!(target: "CiscoIosDevice::close", "Connection closed successfully");
        Ok(())
    }
}

impl CiscoIosDevice {
    pub fn new(config: CiscoDeviceConfig) -> Result<Self, NetsshError> {
        Ok(Self {
            connection: BaseConnection::new()?,
            config,
            prompt: String::new(),
            in_enable_mode: false,
            in_config_mode: false,
        })
    }

    pub fn with_connection(connection: BaseConnection, config: CiscoDeviceConfig) -> Self {
        Self {
            connection,
            config,
            prompt: String::new(),
            in_enable_mode: false,
            in_config_mode: false,
        }
    }

    pub fn connect(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoIosDevice::connect", "Connecting to {}@{}", self.config.username, self.config.host);
        
        self.connection.connect(
            &self.config.host,
            &self.config.username,
            self.config.password.as_deref(),
            self.config.port,
            self.config.timeout,
        )?;

        if let Some(log_file) = &self.config.session_log {
            self.connection.set_session_log(log_file)?;
        }

        self.session_preparation()?;
        Ok(())
    }

    pub fn check_enable_mode(&mut self) -> Result<bool, NetsshError> {
        debug!(target: "CiscoIosDevice::check_enable_mode", "Checking if device is in enable mode");
        
        // Clear any pending data in the buffer
        // let buffer_data = self.clear_buffer(None, None, None)?;
        // debug!(target: "CiscoIosDevice::check_enable_mode", "Cleared buffer data: {:?}", buffer_data);
        
        // Send newline to get prompt
        self.connection.write_channel("\n")?;
        
        // Wait for a short time to ensure the command is processed
        std::thread::sleep(std::time::Duration::from_millis(500));
        
        // Read whatever is available
        let output = match self.connection.read_channel() {
            Ok(out) => out,
            Err(e) => {
                warn!(target: "CiscoIosDevice::check_enable_mode", "Error reading response: {}", e);
                // Assume not in enable mode if we can't read the prompt
                self.in_enable_mode = false;
                return Ok(false);
            }
        };
        
        // Check if any line ends with #
        let is_enable = output.lines().any(|line| line.trim().ends_with("#"));
        self.in_enable_mode = is_enable;
        
        debug!(target: "CiscoIosDevice::check_enable_mode", "Device is in enable mode (#): {}", is_enable);
        Ok(is_enable)
    }

    pub fn enable(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoIosDevice::enable", "Entering enable mode");
        
        // Check if already in enable mode
        if self.check_enable_mode()? {
            debug!(target: "CiscoIosDevice::enable", "Already in enable mode");
            return Ok(());
        }
        
        // Send enable command
        self.connection.write_channel("enable\n")?;
        
        // Wait for password prompt if secret is provided
        if let Some(secret) = &self.config.secret {
            debug!(target: "CiscoIosDevice::enable", "Waiting for password prompt");
            
            // Use a more flexible pattern to match "Password:" with case insensitivity
            let output = self.connection.read_until_pattern("(?i)password")?;
            
            if PASSWORD_PATTERN.is_match(&output) {
                debug!(target: "CiscoIosDevice::enable", "Sending enable password");
                self.connection.write_channel(&format!("{}\n", secret))?;
            } else {
                warn!(target: "CiscoIosDevice::enable", "Password prompt not found in output: {}", output);
                return Err(NetsshError::CommandError("Password prompt not found".to_string()));
            }
        }
        
        // Wait for enable prompt (the # character)
        let output = self.connection.read_until_pattern("#")?;
        
        if !output.trim().ends_with("#") {
            warn!(target: "CiscoIosDevice::enable", "Enable prompt not found");
            return Err(NetsshError::CommandError("Failed to enter enable mode".to_string()));
        }
        
        self.in_enable_mode = true;
        debug!(target: "CiscoIosDevice::enable", "Successfully entered enable mode");
        
        Ok(())
    }

    pub fn exit_enable_mode(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoIosDevice::exit_enable_mode", "Exiting enable mode");
        
        // Check if already not in enable mode
        if !self.check_enable_mode()? {
            debug!(target: "CiscoIosDevice::exit_enable_mode", "Already not in enable mode");
            return Ok(());
        }
        
        // Send disable command
        self.connection.write_channel("disable\n")?;
        
        // Wait for user prompt (the > character)
        let output = self.connection.read_until_pattern(">")?;
        
        if !output.trim().ends_with(">") {
            warn!(target: "CiscoIosDevice::exit_enable_mode", "User prompt not found after disable command");
            return Err(NetsshError::CommandError("Failed to exit enable mode".to_string()));
        }
        
        self.in_enable_mode = false;
        debug!(target: "CiscoIosDevice::exit_enable_mode", "Successfully exited enable mode");
        
        Ok(())
    }
}

#[async_trait]
impl CiscoBaseConnection for CiscoIosDevice {
    fn session_preparation(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoIosDevice::session_preparation", "Preparing session");
        
        // Only open a channel if one doesn't already exist
        if self.connection.channel.is_none() {
            debug!(target: "CiscoIosDevice::session_preparation", "Opening a new channel");
            self.connection.open_channel()?;
        } else {
            debug!(target: "CiscoIosDevice::session_preparation", "Channel already exists, skipping open_channel");
        }
        
        debug!(target: "CiscoIosDevice::session_preparation", "Reading channel");
        // Clear the buffer before starting
        // let buffer_data = self.clear_buffer(None, None, None)?;
        // debug!(target: "CiscoIosDevice::session_preparation", "Cleared buffer data: {:?}", buffer_data);

        debug!(target: "CiscoIosDevice::session_preparation", "Setting base prompt");
        // Set base prompt
        self.set_base_prompt()?;
        
        // Set terminal width
        self.set_terminal_width(511)?;
        
        // Disable paging
        self.disable_paging()?;
        
        
        // Enter enable mode if not already in it
        if !self.check_enable_mode()? {
            debug!(target: "CiscoIosDevice::session_preparation", "Not in privileged mode #, entering enable mode");
            self.enable()?;
        } else {
            debug!(target: "CiscoIosDevice::session_preparation", "Already in privileged mode");
        }
        
        debug!(target: "CiscoIosDevice::session_preparation", "Session preparation completed successfully");
        Ok(())
    }

    fn set_terminal_width(&mut self, width: u32) -> Result<(), NetsshError> {
        debug!(target: "CiscoIosDevice::set_terminal_width", "Setting terminal width to {}", width);
        
        // Clear any pending data in the buffer
        // let buffer_data = self.clear_buffer(None, None, None)?;
        // debug!(target: "CiscoIosDevice::set_terminal_width", "Cleared buffer data: {:?}", buffer_data);
        
        // Send the command with a newline
        let cmd = format!("terminal width {}\n", width);
        self.connection.write_channel(&cmd)?;
        
        // Wait for a short time to ensure the command is processed
        std::thread::sleep(std::time::Duration::from_millis(500));
        
        // Read whatever is available
        let output = match self.connection.read_channel() {
            Ok(out) => out,
            Err(e) => {
                warn!(target: "CiscoIosDevice::set_terminal_width", "Error reading response: {}", e);
                // Continue anyway, don't fail the connection for this
                String::new()
            }
        };
        
        if output.contains("Invalid") || output.contains("Error") {
            warn!(target: "CiscoIosDevice::set_terminal_width", "Error setting terminal width: {}", output);
            // Continue anyway, don't fail the connection for this
        } else {
            debug!(target: "CiscoIosDevice::set_terminal_width", "Terminal width command sent successfully");
        }
        
        // Always return success, even if there was an error
        Ok(())
    }

    fn disable_paging(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoIosDevice::disable_paging", "Disabling paging");
        
        // Clear any pending data in the buffer
        // let buffer_data = self.clear_buffer(None, None, None)?;
        // debug!(target: "CiscoIosDevice::disable_paging", "Cleared buffer data: {:?}", buffer_data);
        
        // Send the command with a newline
        self.connection.write_channel("terminal length 0\n")?;
        
        // Wait for a short time to ensure the command is processed
        std::thread::sleep(std::time::Duration::from_millis(500));
        
        // Read whatever is available
        let output = match self.connection.read_channel() {
            Ok(out) => out,
            Err(e) => {
                warn!(target: "CiscoIosDevice::disable_paging", "Error reading response: {}", e);
                // Continue anyway, don't fail the connection for this
                String::new()
            }
        };
        
        if output.contains("Invalid") || output.contains("Error") {
            warn!(target: "CiscoIosDevice::disable_paging", "Error disabling paging: {}", output);
            // Continue anyway, don't fail the connection for this
        } else {
            debug!(target: "CiscoIosDevice::disable_paging", "Paging disabled successfully");
        }
        
        // Always return success, even if there was an error
        Ok(())
    }

    fn set_base_prompt(&mut self) -> Result<String, NetsshError> {
        debug!(target: "CiscoIosDevice::set_base_prompt", "Setting base prompt");
        
        // Clear any pending data in the buffer
        // let buffer_data = self.clear_buffer(None, None, None)?;
        // debug!(target: "CiscoIosDevice::set_base_prompt", "Cleared buffer data: {:?}", buffer_data);
        
        // Send newline to get prompt
        self.connection.write_channel("\n")?;
        
        // Wait for a short time to ensure the command is processed
        std::thread::sleep(std::time::Duration::from_millis(500));
        
        // Read whatever is available
        let pattern = r"[>#]";
        let output = match self.connection.read_until_pattern(pattern) {
            Ok(out) => out,
            Err(e) => {
                warn!(target: "CiscoIosDevice::set_base_prompt", "Error reading response: {}", e);
                // Use a default prompt if we can't read the actual one
                self.prompt = "Router".to_string();
                // Also set the base_prompt in the BaseConnection
                self.connection.base_prompt = Some(self.prompt.clone());
                // Set the prompt in the SSHChannel
                self.connection.channel.set_base_prompt(&self.prompt);
                return Ok(self.prompt.clone());
            }
        };
        
        // Find the last line that contains a prompt character
        if let Some(last_line) = output.lines().filter(|line| line.contains(">") || line.contains("#")).last() {
            // Extract the prompt without the terminator
            let prompt_end = last_line.find('>').or_else(|| last_line.find('#')).unwrap_or(last_line.len());
            self.prompt = last_line[..prompt_end].trim_end().to_string();
            
            // Cisco IOS abbreviates the prompt at 16 chars in config mode
            if self.prompt.len() > 16 {
                self.prompt = self.prompt[..16].to_string();
            }
            
            debug!(target: "CiscoIosDevice::set_base_prompt", "Base prompt set to: {}", self.prompt);
            
            // Also set the base_prompt in the BaseConnection
            self.connection.base_prompt = Some(self.prompt.clone());
            debug!(target: "CiscoIosDevice::set_base_prompt", "Set base_prompt in BaseConnection to: {}", self.prompt);
            
            // Set the prompt in the SSHChannel
            self.connection.channel.set_base_prompt(&self.prompt);
            debug!(target: "CiscoIosDevice::set_base_prompt", "Set base_prompt in SSHChannel");
        } else {
            // If we can't find a prompt, use a default
            warn!(target: "CiscoIosDevice::set_base_prompt", "Could not find prompt in output: {}", output);
            self.prompt = "Router".to_string();
            
            // Also set the base_prompt in the BaseConnection
            self.connection.base_prompt = Some(self.prompt.clone());
            
            // Set the prompt in the SSHChannel
            self.connection.channel.set_base_prompt(&self.prompt);
        }
        
        Ok(self.prompt.clone())
    }

    fn check_config_mode(&mut self) -> Result<bool, NetsshError> {
        debug!(target: "CiscoIosDevice::check_config_mode", "Checking if device is in config mode");
        
        // Clear any pending data in the buffer
        // let buffer_data = self.clear_buffer(None, None, None)?;
        // debug!(target: "CiscoIosDevice::check_config_mode", "Cleared buffer data: {:?}", buffer_data);
        
        // Send newline to get prompt
        self.connection.write_channel("\n")?;
        
        // Wait for a short time to ensure the command is processed
        std::thread::sleep(std::time::Duration::from_millis(500));
        
        // Read whatever is available
        let output = match self.connection.read_channel() {
            Ok(out) => out,
            Err(e) => {
                warn!(target: "CiscoIosDevice::check_config_mode", "Error reading response: {}", e);
                // Assume not in config mode if we can't read the prompt
                self.in_config_mode = false;
                return Ok(false);
            }
        };
        
        // Check if any line contains the config pattern
        let is_config = output.lines().any(|line| line.contains("(config") && line.contains("#"));
        self.in_config_mode = is_config;
        
        debug!(target: "CiscoIosDevice::check_config_mode", "Device is in config mode: {}", is_config);
        Ok(is_config)
    }

    fn config_mode(&mut self, config_command: Option<&str>) -> Result<(), NetsshError> {
        debug!(target: "CiscoIosDevice::config_mode", "Entering config mode");
        
        // Check if already in config mode
        if self.check_config_mode()? {
            debug!(target: "CiscoIosDevice::config_mode", "Already in config mode");
            return Ok(());
        }
        
        // Ensure we're in enable mode first
        if !self.check_enable_mode()? {
            debug!(target: "CiscoIosDevice::config_mode", "Not in enable mode, entering enable mode first");
            self.enable()?;
        }
        
        // Send config command
        let cmd = config_command.unwrap_or("configure terminal");
        self.connection.write_channel(&format!("{}\n", cmd))?;
        
        // Wait for config prompt
        let output = self.connection.read_until_pattern("\\(config\\)#")?;
        
        if !output.contains("(config)#") {
            warn!(target: "CiscoIosDevice::config_mode", "Config prompt not found after config command: {}", output);
            return Err(NetsshError::CommandError("Failed to enter config mode".to_string()));
        }
        
        self.in_config_mode = true;
        debug!(target: "CiscoIosDevice::config_mode", "Successfully entered config mode");
        
        Ok(())
    }

    fn exit_config_mode(&mut self, exit_command: Option<&str>) -> Result<(), NetsshError> {
        debug!(target: "CiscoIosDevice::exit_config_mode", "Exiting config mode");
        
        // Check if already not in config mode
        if !self.check_config_mode()? {
            debug!(target: "CiscoIosDevice::exit_config_mode", "Already not in config mode");
            return Ok(());
        }
        
        // Send exit command
        let cmd = exit_command.unwrap_or("end");
        self.connection.write_channel(&format!("{}\n", cmd))?;
        
        // Wait for enable prompt
        let output = self.connection.read_until_pattern("#")?;
        
        if !output.trim().ends_with("#") || output.contains("(config)") {
            warn!(target: "CiscoIosDevice::exit_config_mode", "Enable prompt not found after exit command: {}", output);
            return Err(NetsshError::CommandError("Failed to exit config mode".to_string()));
        }
        
        self.in_config_mode = false;
        debug!(target: "CiscoIosDevice::exit_config_mode", "Successfully exited config mode");
        
        Ok(())
    }

    fn save_config(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoIosDevice::save_config", "Saving configuration");
        
        // Ensure we're in enable mode
        if !self.check_enable_mode()? {
            debug!(target: "CiscoIosDevice::save_config", "Not in enable mode, entering enable mode first");
            self.enable()?;
        }
        
        // Exit config mode if we're in it
        if self.check_config_mode()? {
            debug!(target: "CiscoIosDevice::save_config", "In config mode, exiting config mode first");
            self.exit_config_mode(None)?;
        }
        
        // Send save command
        self.connection.write_channel("copy running-config startup-config\n")?;
        
        // Wait for confirmation prompt and send enter
        let output = self.connection.read_until_pattern("\\?")?;
        if output.contains("?") {
            self.connection.write_channel("\n")?;
        }
        
        // Wait for completion
        let output = self.connection.read_until_pattern("#")?;
        
        if output.contains("Error") || !output.contains("OK") {
            warn!(target: "CiscoIosDevice::save_config", "Error saving configuration: {}", output);
            return Err(NetsshError::CommandError(format!("Failed to save configuration: {}", output)));
        }
        
        debug!(target: "CiscoIosDevice::save_config", "Configuration saved successfully");
        Ok(())
    }

    fn send_command(&mut self, command: &str) -> Result<String, NetsshError> {
        debug!(target: "CiscoIosDevice::send_command", "Sending command: {}", command);
        
        // Clear buffer before sending command
        // let buffer_data = self.clear_buffer(None, None, None)?;
        // debug!(target: "CiscoIosDevice::send_command", "Cleared buffer data: {:?}", buffer_data);
        
        // Send command
        self.connection.write_channel(&format!("{}\n", command))?;
        
        // Wait for command echo and prompt
        let pattern = if self.in_config_mode {
            r"\(config.*\)#"
        } else if self.in_enable_mode {
            "#"
        } else {
            ">"
        };
        
        let output = self.connection.read_until_pattern(pattern)?;

        self.connection.session_log.write(&output)?;
        
        // Remove command echo from output
        let lines: Vec<&str> = output.lines().collect();
        let result = if lines.len() > 1 {
            // Skip the first line (command echo) and join the rest
            lines[1..].join("\n")
        } else {
            output
        };
        
        debug!(target: "CiscoIosDevice::send_command", "Command output received, length: {}", result.len());
        Ok(result)
    }
}
