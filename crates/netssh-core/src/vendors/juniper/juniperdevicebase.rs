use crate::base_connection::BaseConnection;
use crate::channel::SSHChannel;
use crate::device_connection::DeviceType;
use crate::error::NetsshError;
use crate::vendors::common::DefaultConfigSetMethods;
use crate::vendors::juniper::{JuniperDeviceConfig, JuniperDeviceConnection};
use async_trait::async_trait;
use lazy_static::lazy_static;
use regex::Regex;
use tracing::{debug, warn};

lazy_static! {
    static ref PROMPT_PATTERN: Regex = Regex::new(r"[>#%]").unwrap();
    static ref CONFIG_PATTERN: Regex = Regex::new(r"#").unwrap();
    static ref PASSWORD_PATTERN: Regex = Regex::new(r"(?i)password").unwrap();
    static ref ANSI_ESCAPE_PATTERN: Regex = Regex::new(r"\x1B\[[0-9;]*[a-zA-Z]").unwrap();
}

pub struct JuniperBaseConnection {
    pub connection: BaseConnection,
    pub config: JuniperDeviceConfig,
    pub prompt: String,
    pub in_config_mode: bool,
}

impl JuniperBaseConnection {
    pub fn new(config: JuniperDeviceConfig) -> Result<Self, NetsshError> {
        let mut connection = BaseConnection::new()?;
        connection.set_device_type(DeviceType::JuniperJunos);

        Ok(Self {
            connection,
            config,
            prompt: String::new(),
            in_config_mode: false,
        })
    }

    pub fn with_connection(mut connection: BaseConnection, config: JuniperDeviceConfig) -> Self {
        // Ensure the connection has the correct device type
        connection.set_device_type(DeviceType::JuniperJunos);

        Self {
            connection,
            config,
            prompt: String::new(),
            in_config_mode: false,
        }
    }

    pub fn connect(&mut self) -> Result<(), NetsshError> {
        debug!(target: "JuniperBaseConnection::connect", "Connecting to {}@{}", self.config.username, self.config.host);

        self.connection.connect(
            Some(&self.config.host),
            Some(&self.config.username),
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

    pub fn close(&mut self) -> Result<(), NetsshError> {
        debug!(target: "JuniperBaseConnection::close", "Closing connection to device");

        // Try to exit config mode if we're in it
        debug!(target: "JuniperBaseConnection::close", "is it in config mode {}", self.in_config_mode);
        if self.in_config_mode {
            let _ = self.exit_config_mode(None);
        }

        // Send exit command to gracefully close the connection
        let _ = self.connection.write_channel("exit\n");

        // Close the channel if it exists
        if let Some(channel) = self.connection.channel.as_mut() {
            debug!(target: "JuniperBaseConnection::close", "Closing SSH channel");
            channel.send_eof().map_err(|e| NetsshError::SshError(e))?;
            channel.wait_eof().map_err(|e| NetsshError::SshError(e))?;
            channel.close().map_err(|e| NetsshError::SshError(e))?;
            channel.wait_close().map_err(|e| NetsshError::SshError(e))?;
        }

        // Clear the channel reference
        self.connection.channel = SSHChannel::new(None);

        // Clear the session reference (which will drop and close the session)
        self.connection.session = None;

        debug!(target: "JuniperBaseConnection::close", "Connection closed successfully");
        Ok(())
    }
}

impl DefaultConfigSetMethods for JuniperBaseConnection {
    fn get_base_connection(&mut self) -> &mut BaseConnection {
        &mut self.connection
    }
}

#[async_trait]
impl JuniperDeviceConnection for JuniperBaseConnection {
    fn session_preparation(&mut self) -> Result<(), NetsshError> {
        debug!(target: "JuniperBaseConnection::session_preparation", "Preparing session");

        // Only open a channel if one doesn't already exist
        if self.connection.channel.is_none() {
            debug!(target: "JuniperBaseConnection::session_preparation", "Opening a new channel");
            self.connection.open_channel()?;
        } else {
            debug!(target: "JuniperBaseConnection::session_preparation", "Channel already exists, skipping open_channel");
        }

        debug!(target: "JuniperBaseConnection::session_preparation", "Setting base prompt");
        // Set base prompt
        self.set_base_prompt()?;

        // Configure terminal settings
        self.terminal_settings()?;

        debug!(target: "JuniperBaseConnection::session_preparation", "Session preparation completed successfully");
        Ok(())
    }

    fn terminal_settings(&mut self) -> Result<(), NetsshError> {
        debug!(target: "JuniperBaseConnection::terminal_settings", "Configuring base terminal settings");

        // Set terminal width
        self.set_terminal_width(511)?;

        // Disable paging
        self.disable_paging()?;

        debug!(target: "JuniperBaseConnection::terminal_settings", "Base terminal settings configured successfully");
        Ok(())
    }

    fn set_terminal_width(&mut self, width: u32) -> Result<(), NetsshError> {
        debug!(target: "JuniperBaseConnection::set_terminal_width", "Setting terminal width to {}", width);

        // Send the command with a newline
        let cmd = format!("set cli screen-width {}\n", width);
        self.connection.write_channel(&cmd)?;

        // Wait for a short time to ensure the command is processed
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Read whatever is available
        let output = match self.connection.read_channel() {
            Ok(out) => out,
            Err(e) => {
                warn!(target: "JuniperBaseConnection::set_terminal_width", "Error reading response: {}", e);
                // Continue anyway, don't fail the connection for this
                String::new()
            }
        };

        if output.contains("Invalid") || output.contains("Error") {
            warn!(target: "JuniperBaseConnection::set_terminal_width", "Error setting terminal width: {}", output);
            // Continue anyway, don't fail the connection for this
        } else {
            debug!(target: "JuniperBaseConnection::set_terminal_width", "Terminal width command sent successfully");
        }

        // Always return success, even if there was an error
        Ok(())
    }

    fn disable_paging(&mut self) -> Result<(), NetsshError> {
        debug!(target: "JuniperBaseConnection::disable_paging", "Disabling paging");

        // Send the command with a newline - Juniper uses "set cli screen-length 0"
        self.connection.write_channel("set cli screen-length 0\n")?;

        // Wait for a short time to ensure the command is processed
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Read whatever is available
        let output = match self.connection.read_channel() {
            Ok(out) => out,
            Err(e) => {
                warn!(target: "JuniperBaseConnection::disable_paging", "Error reading response: {}", e);
                // Continue anyway, don't fail the connection for this
                String::new()
            }
        };

        if output.contains("Invalid") || output.contains("Error") {
            warn!(target: "JuniperBaseConnection::disable_paging", "Error disabling paging: {}", output);
            // Continue anyway, don't fail the connection for this
        } else {
            debug!(target: "JuniperBaseConnection::disable_paging", "Paging disabled successfully");
        }

        // Always return success, even if there was an error
        Ok(())
    }

    fn set_base_prompt(&mut self) -> Result<String, NetsshError> {
        debug!(target: "JuniperBaseConnection::set_base_prompt", "Setting base prompt");

        // Send newline to get prompt
        self.connection.write_channel("\n")?;

        // Wait for a short time to ensure the command is processed
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Read whatever is available
        let pattern = r"[>#%]";
        let output = match self.connection.read_until_pattern(pattern, None, None) {
            Ok(out) => out,
            Err(e) => {
                warn!(target: "JuniperBaseConnection::set_base_prompt", "Error reading response: {}", e);
                // Use a default prompt if we can't read the actual one
                self.prompt = "juniper".to_string();
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
            .filter(|line| line.contains(">") || line.contains("#") || line.contains("%"))
            .last()
        {
            // Extract the prompt without the terminator
            let prompt_end = last_line
                .find('>')
                .or_else(|| last_line.find('#'))
                .or_else(|| last_line.find('%'))
                .unwrap_or(last_line.len());
            self.prompt = last_line[..prompt_end].trim_end().to_string();

            debug!(target: "JuniperBaseConnection::set_base_prompt", "Base prompt set to: {}", self.prompt);

            // Also set the base_prompt in the BaseConnection
            self.connection.base_prompt = Some(self.prompt.clone());
            debug!(target: "JuniperBaseConnection::set_base_prompt", "Set base_prompt in BaseConnection to: {}", self.prompt);

            // Set the prompt in the SSHChannel
            self.connection.channel.set_base_prompt(&self.prompt);
            debug!(target: "JuniperBaseConnection::set_base_prompt", "Set base_prompt in SSHChannel");
        } else {
            // If we can't find a prompt, use a default
            warn!(target: "JuniperBaseConnection::set_base_prompt", "Could not find prompt in output: {}", output);
            self.prompt = "juniper".to_string();

            // Also set the base_prompt in the BaseConnection
            self.connection.base_prompt = Some(self.prompt.clone());

            // Set the prompt in the SSHChannel
            self.connection.channel.set_base_prompt(&self.prompt);
        }

        Ok(self.prompt.clone())
    }

    fn check_config_mode(&mut self) -> Result<bool, NetsshError> {
        debug!(target: "JuniperBaseConnection::check_config_mode", "Checking if device is in config mode");

        // Send newline to get prompt
        self.connection.write_channel("\n")?;

        // Wait for a short time to ensure the command is processed
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Read whatever is available
        let output = match self.connection.read_channel() {
            Ok(out) => out,
            Err(e) => {
                warn!(target: "JuniperBaseConnection::check_config_mode", "Error reading response: {}", e);
                // Assume not in config mode if we can't read the prompt
                self.in_config_mode = false;
                return Ok(false);
            }
        };

        // Check if any line contains the config pattern (# at the end)
        let is_config = output.lines().any(|line| line.trim().ends_with("#"));
        self.in_config_mode = is_config;

        debug!(target: "JuniperBaseConnection::check_config_mode", "Device is in config mode: {}", is_config);
        Ok(is_config)
    }

    fn config_mode(&mut self, config_command: Option<&str>) -> Result<(), NetsshError> {
        debug!(target: "JuniperBaseConnection::config_mode", "Entering config mode");

        // Check if already in config mode
        if self.check_config_mode()? {
            debug!(target: "JuniperBaseConnection::config_mode", "Already in config mode");
            return Ok(());
        }

        // Send config command
        let cmd = config_command.unwrap_or("configure");
        self.connection.write_channel(&format!("{}\n", cmd))?;

        // Wait for config prompt
        let output = self.connection.read_until_pattern("#", None, None)?;

        if !output.contains("#") {
            warn!(target: "JuniperBaseConnection::config_mode", "Config prompt not found after config command: {}", output);
            return Err(NetsshError::CommandError(
                "Failed to enter config mode".to_string(),
            ));
        }

        self.in_config_mode = true;
        debug!(target: "JuniperBaseConnection::config_mode", "Successfully entered config mode");

        Ok(())
    }

    fn exit_config_mode(&mut self, exit_command: Option<&str>) -> Result<(), NetsshError> {
        debug!(target: "JuniperBaseConnection::exit_config_mode", "Exiting config mode");

        // Check if already not in config mode
        if !self.check_config_mode()? {
            debug!(target: "JuniperBaseConnection::exit_config_mode", "Already not in config mode");
            return Ok(());
        }

        // Send exit command
        let cmd = exit_command.unwrap_or("exit configuration-mode");
        self.connection.write_channel(&format!("{}\n", cmd))?;

        // Wait for operational prompt
        let output = self.connection.read_until_pattern(">", None, None)?;

        if !output.trim().ends_with(">") {
            warn!(target: "JuniperBaseConnection::exit_config_mode", "Operational prompt not found after exit command: {}", output);
            return Err(NetsshError::CommandError(
                "Failed to exit config mode".to_string(),
            ));
        }

        self.in_config_mode = false;
        debug!(target: "JuniperBaseConnection::exit_config_mode", "Successfully exited config mode");

        Ok(())
    }

    fn commit_config(&mut self) -> Result<String, NetsshError> {
        debug!(target: "JuniperBaseConnection::commit_config", "Committing configuration");

        // Check if in config mode
        if !self.check_config_mode()? {
            debug!(target: "JuniperBaseConnection::commit_config", "Not in config mode, entering config mode first");
            self.config_mode(None)?;
        }

        // Send commit command
        self.connection.write_channel("commit\n")?;

        // Wait for completion
        let output = self
            .connection
            .read_until_pattern(&self.prompt, None, None)?;

        // Simple error check based on known patterns
        if output.contains("error") || output.contains("failed") {
            // Filter out known success messages
            if !output.contains("commit complete") && !output.contains("configuration not changed")
            {
                warn!(target: "JuniperBaseConnection::commit_config", "Error committing configuration: {}", output);
                return Err(NetsshError::command_error_with_output(
                    format!("Failed to commit configuration"),
                    output,
                ));
            }
        }

        debug!(target: "JuniperBaseConnection::commit_config", "Configuration committed successfully");
        Ok(output)
    }

    fn send_command(
        &mut self,
        command: &str,
        expect_string: Option<&str>,
        read_timeout: Option<f64>,
        auto_find_prompt: Option<bool>,
        strip_prompt: Option<bool>,
        strip_command: Option<bool>,
        normalize: Option<bool>,
        cmd_verify: Option<bool>,
    ) -> Result<String, NetsshError> {
        debug!(target: "JuniperBaseConnection::send_command", "Sending command: {}", command);

        // Use self.prompt as expect_string if it's not empty, otherwise use the provided expect_string
        let effective_expect_string = if !self.prompt.is_empty() {
            Some(self.prompt.as_str())
        } else {
            expect_string
        };

        // Call an inherent method or directly forward to connection to avoid recursion
        self.connection.send_command(
            command,
            effective_expect_string,
            read_timeout,
            auto_find_prompt,
            strip_prompt,
            strip_command,
            normalize,
            cmd_verify,
        )
    }
}
