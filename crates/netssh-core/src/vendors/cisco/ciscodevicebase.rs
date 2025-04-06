use crate::base_connection::BaseConnection;
use crate::channel::SSHChannel;
use crate::error::NetsshError;
use crate::vendors::cisco::{CiscoDeviceConfig, CiscoDeviceConnection};
use crate::vendors::common::DefaultConfigSetMethods;
use lazy_static::lazy_static;
use regex::Regex;
use tracing::{debug, warn};

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

        let host = self.config.host.as_str();
        let username = self.config.username.as_str();

        self.connection.connect(
            Some(host),
            Some(username),
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

        let result = self.connection.check_enable_mode(Some("#"))?;

        self.in_enable_mode = result;

        debug!(target: "CiscoBaseConnection::check_enable_mode", "Device is in enable mode: {}", result);
        Ok(result)
    }

    pub fn enable(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoBaseConnection::enable", "Entering enable mode");

        if self.check_enable_mode()? {
            debug!(target: "CiscoBaseConnection::enable", "Already in enable mode");
            return Ok(());
        }

        let cmd = "enable";
        let pattern = "assword";
        let secret = self.config.secret.as_deref();

        if let Some(_secret_str) = secret {
            let output =
                self.connection
                    .enable(Some(cmd), Some(pattern), None, Some(true), None)?;

            debug!(target: "CiscoBaseConnection::enable", "Enable output: {}", output);

            self.in_enable_mode = true;
        } else {
            self.connection.write_channel("enable\n")?;
            let output = self.connection.read_until_pattern("#", None, None)?;

            if !output.trim().ends_with("#") {
                return Err(NetsshError::CommandError(
                    "Failed to enter enable mode".to_string(),
                ));
            }

            self.in_enable_mode = true;
        }

        debug!(target: "CiscoBaseConnection::enable", "Successfully entered enable mode");
        Ok(())
    }

    pub fn exit_enable_mode(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoBaseConnection::exit_enable_mode", "Exiting enable mode");

        if !self.check_enable_mode()? {
            debug!(target: "CiscoBaseConnection::exit_enable_mode", "Already not in enable mode");
            return Ok(());
        }

        let output = self.connection.exit_enable_mode(Some("disable"))?;
        debug!(target: "CiscoBaseConnection::exit_enable_mode", "Exit enable mode output: {}", output);

        self.in_enable_mode = false;
        debug!(target: "CiscoBaseConnection::exit_enable_mode", "Successfully exited enable mode");

        Ok(())
    }

    pub fn close(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoBaseConnection::close", "Closing connection to device");

        if self.in_config_mode {
            let _ = self.exit_config_mode(None);
        }

        let _ = self.connection.write_channel("exit\n");

        if let Some(channel) = self.connection.channel.as_mut() {
            debug!(target: "CiscoBaseConnection::close", "Closing SSH channel");
            channel.send_eof().map_err(|e| NetsshError::SshError(e))?;
            channel.wait_eof().map_err(|e| NetsshError::SshError(e))?;
            channel.close().map_err(|e| NetsshError::SshError(e))?;
            channel.wait_close().map_err(|e| NetsshError::SshError(e))?;
        }

        self.connection.channel = SSHChannel::new(None);

        self.connection.session = None;

        debug!(target: "CiscoBaseConnection::close", "Connection closed successfully");
        Ok(())
    }

    pub fn session_preparation(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoBaseConnection::session_preparation", "Preparing session");

        if self.connection.channel.is_none() {
            debug!(target: "CiscoBaseConnection::session_preparation", "Opening a new channel");
            self.connection.open_channel()?;
        } else {
            debug!(target: "CiscoBaseConnection::session_preparation", "Channel already exists, skipping open_channel");
        }

        let output = self
            .connection
            .clear_buffer(None, Some("[>#]"), None, None, None, None)?;
        debug!(target: "CiscoBaseConnection::session_preparation", "Channel read test completed: {}", output);

        self.set_base_prompt()?;

        self.terminal_settings()?;

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

        self.set_terminal_width(511)?;

        self.disable_paging()?;

        debug!(target: "CiscoBaseConnection::terminal_settings", "Base terminal settings configured successfully");
        Ok(())
    }

    pub fn set_terminal_width(&mut self, width: u32) -> Result<(), NetsshError> {
        debug!(target: "CiscoBaseConnection::set_terminal_width", "Setting terminal width to {}", width);

        let cmd = format!("terminal width {}\n", width);
        self.connection.write_channel(&cmd)?;

        std::thread::sleep(std::time::Duration::from_millis(500));

        let output = match self.connection.read_channel() {
            Ok(out) => out,
            Err(e) => {
                warn!(target: "CiscoBaseConnection::set_terminal_width", "Error reading response: {}", e);
                String::new()
            }
        };

        if output.contains("Invalid") || output.contains("Error") {
            warn!(target: "CiscoBaseConnection::set_terminal_width", "Error setting terminal width: {}", output);
        } else {
            debug!(target: "CiscoBaseConnection::set_terminal_width", "Terminal width command sent successfully");
        }

        Ok(())
    }

    pub fn disable_paging(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoBaseConnection::disable_paging", "Disabling paging");

        self.connection.write_channel("terminal length 0\n")?;

        std::thread::sleep(std::time::Duration::from_millis(500));

        let output = match self.connection.read_channel() {
            Ok(out) => out,
            Err(e) => {
                warn!(target: "CiscoBaseConnection::disable_paging", "Error reading response: {}", e);
                String::new()
            }
        };

        if output.contains("Invalid") || output.contains("Error") {
            warn!(target: "CiscoBaseConnection::disable_paging", "Error disabling paging: {}", output);
        } else {
            debug!(target: "CiscoBaseConnection::disable_paging", "Paging disabled successfully");
        }

        Ok(())
    }

    pub fn set_base_prompt(&mut self) -> Result<String, NetsshError> {
        debug!(target: "CiscoBaseConnection::set_base_prompt", "Setting base prompt");

        // Call the base implementation with Cisco IOS specific defaults
        let prompt = self.connection.set_base_prompt(
            Some("#"), // Primary prompt terminator
            Some(">"), // Alternate prompt terminator
            None,      // Use default delay factor
            None,      // Use default pattern
        )?;

        // Store the prompt in our local state
        self.prompt = prompt.clone();

        // Set the prompt in the channel (this is handled by base implementation but we keep it for compatibility)
        self.connection.channel.set_base_prompt(&self.prompt);

        Ok(prompt)
    }

    pub fn check_config_mode(&mut self) -> Result<bool, NetsshError> {
        debug!(target: "CiscoBaseConnection::check_config_mode", "Checking if device is in config mode");

        let result = self
            .connection
            .check_config_mode(Some(")#"), Some("[>#]"), None)?;

        self.in_config_mode = result;

        debug!(target: "CiscoBaseConnection::check_config_mode", "Device is in config mode: {}", result);
        Ok(result)
    }

    pub fn config_mode(&mut self, config_command: Option<&str>) -> Result<(), NetsshError> {
        debug!(target: "CiscoBaseConnection::config_mode", "Entering config mode");

        if !self.check_enable_mode()? {
            debug!(target: "CiscoBaseConnection::config_mode", "Not in enable mode, entering enable mode first");
            self.enable()?;
        }

        if self.check_config_mode()? {
            debug!(target: "CiscoBaseConnection::config_mode", "Already in config mode");
            return Ok(());
        }

        let cmd = config_command.unwrap_or("configure terminal");
        let output = self.connection.config_mode(Some(cmd), None, None)?;
        debug!(target: "CiscoBaseConnection::config_mode", "Config mode output: {}", output);

        self.in_config_mode = true;
        debug!(target: "CiscoBaseConnection::config_mode", "Successfully entered config mode");

        Ok(())
    }

    pub fn exit_config_mode(&mut self, exit_command: Option<&str>) -> Result<(), NetsshError> {
        debug!(target: "CiscoBaseConnection::exit_config_mode", "Exiting config mode");

        if !self.check_config_mode()? {
            debug!(target: "CiscoBaseConnection::exit_config_mode", "Already not in config mode");
            return Ok(());
        }

        let cmd = exit_command.unwrap_or("end");
        let pattern = r"#.*";
        let output = self.connection.exit_config_mode(Some(cmd), Some(pattern))?;
        debug!(target: "CiscoBaseConnection::exit_config_mode", "Exit config mode output: {}", output);

        self.in_config_mode = false;
        debug!(target: "CiscoBaseConnection::exit_config_mode", "Successfully exited config mode");

        Ok(())
    }

    pub fn save_config(&mut self) -> Result<String, NetsshError> {
        debug!(target: "CiscoBaseConnection::save_config", "Saving configuration");

        if !self.check_enable_mode()? {
            debug!(target: "CiscoBaseConnection::save_config", "Not in enable mode, entering enable mode first");
            self.enable()?;
        }

        let output = self.connection.save_config(
            Some("copy running-config startup-config"),
            Some(true),
            None,
        )?;

        debug!(target: "CiscoBaseConnection::save_config", "Configuration saved successfully");
        Ok(output)
    }

    pub fn send_command(&mut self, command: &str) -> Result<String, NetsshError> {
        debug!(target: "CiscoBaseConnection::send_command", "Sending command: {}", command);

        let output = self.connection.send_command(
            command,
            None,
            None,
            Some(true),
            Some(true),
            Some(true),
            Some(true),
            Some(true),
        )?;

        debug!(target: "CiscoBaseConnection::send_command", "Command output received, length: {}", output.len());
        Ok(output)
    }
}

impl DefaultConfigSetMethods for CiscoBaseConnection {
    fn get_base_connection(&mut self) -> &mut BaseConnection {
        &mut self.connection
    }
}

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

    fn save_config(&mut self) -> Result<String, NetsshError> {
        self.save_config()
    }

    fn send_command(&mut self, command: &str) -> Result<String, NetsshError> {
        self.send_command(command)
    }
}
