use crate::base_connection::BaseConnection;
use crate::channel::SSHChannel;
use crate::error::NetsshError;
use crate::vendors::cisco::{CiscoDeviceConfig, CiscoDeviceConnection};
use async_trait::async_trait;
use lazy_static::lazy_static;
use log::{debug, warn};
use regex::Regex;

lazy_static! {
    static ref PROMPT_PATTERN: Regex = Regex::new(r"[>#]").unwrap();
    static ref CONFIG_PATTERN: Regex = Regex::new(r"\)#").unwrap();
    static ref PASSWORD_PATTERN: Regex = Regex::new(r"(?i)password").unwrap();
    static ref ANSI_ESCAPE_PATTERN: Regex = Regex::new(r"\x1B\[[0-9;]*[a-zA-Z]").unwrap();
}

pub struct CiscoBaseConnection {
    pub connection: BaseConnection,
    pub config: CiscoDeviceConfig,
    pub prompt: String,
    pub in_enable_mode: bool,
    pub in_config_mode: bool,
}

impl CiscoBaseConnection {
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
        debug!(target: "CiscoBaseConnection::connect", "Connecting to {}@{}", self.config.username, self.config.host);

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

    /// Strip ANSI escape codes from a string
    pub fn strip_ansi_escape_codes(&self, data: &str) -> String {
        ANSI_ESCAPE_PATTERN.replace_all(data, "").to_string()
    }

    pub fn check_enable_mode(&mut self) -> Result<bool, NetsshError> {
        debug!(target: "CiscoBaseConnection::check_enable_mode", "Checking if device is in enable mode");

        // Send newline to get prompt
        self.connection.write_channel("\n")?;

        // Wait for a short time to ensure the command is processed
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Read whatever is available
        let output = match self.connection.read_channel() {
            Ok(out) => out,
            Err(e) => {
                warn!(target: "CiscoBaseConnection::check_enable_mode", "Error reading response: {}", e);
                // Assume not in enable mode if we can't read the prompt
                self.in_enable_mode = false;
                return Ok(false);
            }
        };

        // Check if any line ends with #
        let is_enable = output.lines().any(|line| line.trim().ends_with("#"));
        self.in_enable_mode = is_enable;

        debug!(target: "CiscoBaseConnection::check_enable_mode", "Device is in enable mode (#): {}", is_enable);
        Ok(is_enable)
    }

    pub fn enable(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoBaseConnection::enable", "Entering enable mode");

        // Check if already in enable mode
        if self.check_enable_mode()? {
            debug!(target: "CiscoBaseConnection::enable", "Already in enable mode");
            return Ok(());
        }

        // Send enable command
        self.connection.write_channel("enable\n")?;

        // Wait for password prompt if secret is provided
        if let Some(secret) = &self.config.secret {
            debug!(target: "CiscoBaseConnection::enable", "Waiting for password prompt");

            // Use a more flexible pattern to match "Password:" with case insensitivity
            let output = self
                .connection
                .read_until_pattern("(?i)password", None, None)?;

            if PASSWORD_PATTERN.is_match(&output) {
                debug!(target: "CiscoBaseConnection::enable", "Sending enable password");
                self.connection.write_channel(&format!("{}\n", secret))?;
            } else {
                warn!(target: "CiscoBaseConnection::enable", "Password prompt not found in output: {}", output);
                return Err(NetsshError::CommandError(
                    "Password prompt not found".to_string(),
                ));
            }
        }

        // Wait for enable prompt (the # character)
        let output = self.connection.read_until_pattern("#", None, None)?;

        if !output.trim().ends_with("#") {
            warn!(target: "CiscoBaseConnection::enable", "Enable prompt not found");
            return Err(NetsshError::CommandError(
                "Failed to enter enable mode".to_string(),
            ));
        }

        self.in_enable_mode = true;
        debug!(target: "CiscoBaseConnection::enable", "Successfully entered enable mode");

        Ok(())
    }

    pub fn exit_enable_mode(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoBaseConnection::exit_enable_mode", "Exiting enable mode");

        // Check if already not in enable mode
        if !self.check_enable_mode()? {
            debug!(target: "CiscoBaseConnection::exit_enable_mode", "Already not in enable mode");
            return Ok(());
        }

        // Send disable command
        self.connection.write_channel("disable\n")?;

        // Wait for user prompt (the > character)
        let output = self.connection.read_until_pattern(">", None, None)?;

        if !output.trim().ends_with(">") {
            warn!(target: "CiscoBaseConnection::exit_enable_mode", "User prompt not found after disable command");
            return Err(NetsshError::CommandError(
                "Failed to exit enable mode".to_string(),
            ));
        }

        self.in_enable_mode = false;
        debug!(target: "CiscoBaseConnection::exit_enable_mode", "Successfully exited enable mode");

        Ok(())
    }

    pub fn close(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoBaseConnection::close", "Closing connection to device");

        // Try to exit config mode if we're in it
        debug!(target: "CiscoBaseConnection::close", "is it in config mode {}", self.in_config_mode);
        if self.in_config_mode {
            let _ = self.exit_config_mode(None);
        }

        // Send exit command to gracefully close the connection
        let _ = self.connection.write_channel("exit\n");

        // Close the channel if it exists
        if let Some(channel) = self.connection.channel.as_mut() {
            debug!(target: "CiscoBaseConnection::close", "Closing SSH channel");
            channel.send_eof().map_err(|e| NetsshError::SshError(e))?;
            channel.wait_eof().map_err(|e| NetsshError::SshError(e))?;
            channel.close().map_err(|e| NetsshError::SshError(e))?;
            channel.wait_close().map_err(|e| NetsshError::SshError(e))?;
        }

        // Clear the channel reference
        self.connection.channel = SSHChannel::new(None);

        // Clear the session reference (which will drop and close the session)
        self.connection.session = None;

        debug!(target: "CiscoBaseConnection::close", "Connection closed successfully");
        Ok(())
    }

    pub fn session_preparation(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoBaseConnection::session_preparation", "Preparing session");

        // Only open a channel if one doesn't already exist
        if self.connection.channel.is_none() {
            debug!(target: "CiscoBaseConnection::session_preparation", "Opening a new channel");
            self.connection.open_channel()?;
        } else {
            debug!(target: "CiscoBaseConnection::session_preparation", "Channel already exists, skipping open_channel");
        }

        debug!(target: "CiscoBaseConnection::session_preparation", "Setting base prompt");

        let output = self.connection.clear_buffer(Some("[>#]"), Some(20), None)?;
        debug!(target: "CiscoBaseConnection::session_preparation", "Cleared buffer: {}", output);
        // Set base prompt
        self.set_base_prompt()?;

        // Call terminal_settings which can be overridden by device-specific implementations
        self.terminal_settings()?;

        // Enter enable mode if not already in it
        if !self.check_enable_mode()? {
            debug!(target: "CiscoBaseConnection::session_preparation", "Not in privileged mode #, entering enable mode");
            self.enable()?;
        } else {
            debug!(target: "CiscoBaseConnection::session_preparation", "Already in privileged mode");
        }

        debug!(target: "CiscoBaseConnection::session_preparation", "Session preparation completed successfully");
        Ok(())
    }

    /// Configure terminal settings - can be overridden by device-specific implementations
    pub fn terminal_settings(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoBaseConnection::terminal_settings", "Configuring base terminal settings");

        // Set terminal width
        self.set_terminal_width(511)?;

        // Disable paging
        self.disable_paging()?;

        debug!(target: "CiscoBaseConnection::terminal_settings", "Base terminal settings configured successfully");
        Ok(())
    }

    pub fn set_terminal_width(&mut self, width: u32) -> Result<(), NetsshError> {
        debug!(target: "CiscoBaseConnection::set_terminal_width", "Setting terminal width to {}", width);

        // Send the command with a newline
        let cmd = format!("terminal width {}\n", width);
        self.connection.write_channel(&cmd)?;

        // Wait for a short time to ensure the command is processed
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Read whatever is available
        let output = match self.connection.read_channel() {
            Ok(out) => out,
            Err(e) => {
                warn!(target: "CiscoBaseConnection::set_terminal_width", "Error reading response: {}", e);
                // Continue anyway, don't fail the connection for this
                String::new()
            }
        };

        if output.contains("Invalid") || output.contains("Error") {
            warn!(target: "CiscoBaseConnection::set_terminal_width", "Error setting terminal width: {}", output);
            // Continue anyway, don't fail the connection for this
        } else {
            debug!(target: "CiscoBaseConnection::set_terminal_width", "Terminal width command sent successfully");
        }

        // Always return success, even if there was an error
        Ok(())
    }

    pub fn disable_paging(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoBaseConnection::disable_paging", "Disabling paging");

        // Send the command with a newline - default is "terminal length 0" for most Cisco devices
        self.connection.write_channel("terminal length 0\n")?;

        // Wait for a short time to ensure the command is processed
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Read whatever is available
        let output = match self.connection.read_channel() {
            Ok(out) => out,
            Err(e) => {
                warn!(target: "CiscoBaseConnection::disable_paging", "Error reading response: {}", e);
                // Continue anyway, don't fail the connection for this
                String::new()
            }
        };

        if output.contains("Invalid") || output.contains("Error") {
            warn!(target: "CiscoBaseConnection::disable_paging", "Error disabling paging: {}", output);
            // Continue anyway, don't fail the connection for this
        } else {
            debug!(target: "CiscoBaseConnection::disable_paging", "Paging disabled successfully");
        }

        // Always return success, even if there was an error
        Ok(())
    }

    pub fn set_base_prompt(&mut self) -> Result<String, NetsshError> {
        debug!(target: "CiscoBaseConnection::set_base_prompt", "Setting base prompt");

        // Send newline to get prompt
        self.connection.write_channel("\n")?;

        // Wait for a short time to ensure the command is processed
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Read whatever is available
        let pattern = r"[>#]";
        let output = match self.connection.read_until_pattern(pattern, None, None) {
            Ok(out) => out,
            Err(e) => {
                warn!(target: "CiscoBaseConnection::set_base_prompt", "Error reading response: {}", e);
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
        if let Some(last_line) = output
            .lines()
            .filter(|line| line.contains(">") || line.contains("#"))
            .last()
        {
            // Extract the prompt without the terminator
            let prompt_end = last_line
                .find('>')
                .or_else(|| last_line.find('#'))
                .unwrap_or(last_line.len());
            self.prompt = last_line[..prompt_end].trim_end().to_string();

            debug!(target: "CiscoBaseConnection::set_base_prompt", "Base prompt set to: {}", self.prompt);

            // Also set the base_prompt in the BaseConnection
            self.connection.base_prompt = Some(self.prompt.clone());
            debug!(target: "CiscoBaseConnection::set_base_prompt", "Set base_prompt in BaseConnection to: {}", self.prompt);

            // Set the prompt in the SSHChannel
            self.connection.channel.set_base_prompt(&self.prompt);
            debug!(target: "CiscoBaseConnection::set_base_prompt", "Set base_prompt in SSHChannel");
        } else {
            // If we can't find a prompt, use a default
            warn!(target: "CiscoBaseConnection::set_base_prompt", "Could not find prompt in output: {}", output);
            self.prompt = "Router".to_string();

            // Also set the base_prompt in the BaseConnection
            self.connection.base_prompt = Some(self.prompt.clone());

            // Set the prompt in the SSHChannel
            self.connection.channel.set_base_prompt(&self.prompt);
        }

        Ok(self.prompt.clone())
    }

    pub fn check_config_mode(&mut self) -> Result<bool, NetsshError> {
        debug!(target: "CiscoBaseConnection::check_config_mode", "Checking if device is in config mode");

        // Send newline to get prompt
        self.connection.write_channel("\n")?;

        // Wait for a short time to ensure the command is processed
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Read whatever is available
        let output = match self.connection.read_channel() {
            Ok(out) => out,
            Err(e) => {
                warn!(target: "CiscoBaseConnection::check_config_mode", "Error reading response: {}", e);
                // Assume not in config mode if we can't read the prompt
                self.in_config_mode = false;
                return Ok(false);
            }
        };

        // Check if any line contains the config pattern
        let is_config = output
            .lines()
            .any(|line| line.contains("(config") && line.contains("#"));
        self.in_config_mode = is_config;

        debug!(target: "CiscoBaseConnection::check_config_mode", "Device is in config mode: {}", is_config);
        Ok(is_config)
    }

    pub fn config_mode(&mut self, config_command: Option<&str>) -> Result<(), NetsshError> {
        debug!(target: "CiscoBaseConnection::config_mode", "Entering config mode");

        // Check if already in config mode
        if self.check_config_mode()? {
            debug!(target: "CiscoBaseConnection::config_mode", "Already in config mode");
            return Ok(());
        }

        // Ensure we're in enable mode first
        if !self.check_enable_mode()? {
            debug!(target: "CiscoBaseConnection::config_mode", "Not in enable mode, entering enable mode first");
            self.enable()?;
        }

        // Send config command
        let cmd = config_command.unwrap_or("configure terminal");
        self.connection.write_channel(&format!("{}\n", cmd))?;

        // Wait for config prompt
        let output = self
            .connection
            .read_until_pattern("\\(config\\)#", None, None)?;

        if !output.contains("(config)#") {
            warn!(target: "CiscoBaseConnection::config_mode", "Config prompt not found after config command: {}", output);
            return Err(NetsshError::CommandError(
                "Failed to enter config mode".to_string(),
            ));
        }

        self.in_config_mode = true;
        debug!(target: "CiscoBaseConnection::config_mode", "Successfully entered config mode");

        Ok(())
    }

    pub fn exit_config_mode(&mut self, exit_command: Option<&str>) -> Result<(), NetsshError> {
        debug!(target: "CiscoBaseConnection::exit_config_mode", "Exiting config mode");

        // Check if already not in config mode
        if !self.check_config_mode()? {
            debug!(target: "CiscoBaseConnection::exit_config_mode", "Already not in config mode");
            return Ok(());
        }

        // Send exit command
        let cmd = exit_command.unwrap_or("end");
        self.connection.write_channel(&format!("{}\n", cmd))?;

        // Wait for enable prompt
        let output = self.connection.read_until_pattern("#", None, None)?;

        if !output.trim().ends_with("#") || output.contains("(config)") {
            warn!(target: "CiscoBaseConnection::exit_config_mode", "Enable prompt not found after exit command: {}", output);
            return Err(NetsshError::CommandError(
                "Failed to exit config mode".to_string(),
            ));
        }

        self.in_config_mode = false;
        debug!(target: "CiscoBaseConnection::exit_config_mode", "Successfully exited config mode");

        Ok(())
    }

    pub fn save_config(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoBaseConnection::save_config", "Saving configuration");

        // Ensure we're in enable mode
        if !self.check_enable_mode()? {
            debug!(target: "CiscoBaseConnection::save_config", "Not in enable mode, entering enable mode first");
            self.enable()?;
        }

        // Exit config mode if we're in it
        if self.check_config_mode()? {
            debug!(target: "CiscoBaseConnection::save_config", "In config mode, exiting config mode first");
            self.exit_config_mode(None)?;
        }

        // Send save command - default for IOS/NXOS
        self.connection.write_channel("write mem\n")?;

        // Wait for completion
        let output = self
            .connection
            .read_until_pattern(&self.prompt, None, None)?;

        debug!(target: "CiscoBaseConnection::save_config", "Save command output: {}", output);

        if output.contains("Error") {
            warn!(target: "CiscoBaseConnection::save_config", "Error saving configuration: {}", output);
            return Err(NetsshError::CommandError(format!(
                "Failed to save configuration: {}",
                output
            )));
        }

        debug!(target: "CiscoBaseConnection::save_config", "Configuration saved successfully");
        Ok(())
    }

    pub fn send_command(&mut self, command: &str) -> Result<String, NetsshError> {
        debug!(target: "CiscoBaseConnection::send_command", "Sending command: {}", command);

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

        let output = self.connection.read_until_pattern(pattern, None, None)?;

        self.connection.session_log.write(&output)?;

        // Remove command echo from output
        let lines: Vec<&str> = output.lines().collect();
        let result = if lines.len() > 2 {
            // Skip the first line (command echo) and last line (prompt), join the rest
            lines[1..lines.len() - 1].join("\n")
        } else {
            output.to_string()
        };

        debug!(target: "CiscoBaseConnection::send_command", "Command output received, length: {}", result.len());
        Ok(result)
    }
}

#[async_trait]
impl CiscoDeviceConnection for CiscoBaseConnection {
    fn session_preparation(&mut self) -> Result<(), NetsshError> {
        self.session_preparation()
    }

    fn terminal_settings(&mut self) -> Result<(), NetsshError> {
        self.terminal_settings()
    }

    fn set_terminal_width(&mut self, width: u32) -> Result<(), NetsshError> {
        self.set_terminal_width(width)
    }

    fn disable_paging(&mut self) -> Result<(), NetsshError> {
        self.disable_paging()
    }

    fn set_base_prompt(&mut self) -> Result<String, NetsshError> {
        self.set_base_prompt()
    }

    fn check_config_mode(&mut self) -> Result<bool, NetsshError> {
        self.check_config_mode()
    }

    fn config_mode(&mut self, config_command: Option<&str>) -> Result<(), NetsshError> {
        self.config_mode(config_command)
    }

    fn exit_config_mode(&mut self, exit_command: Option<&str>) -> Result<(), NetsshError> {
        self.exit_config_mode(exit_command)
    }

    fn save_config(&mut self) -> Result<(), NetsshError> {
        self.save_config()
    }

    fn send_command(&mut self, command: &str) -> Result<String, NetsshError> {
        self.send_command(command)
    }
}
