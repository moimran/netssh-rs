use crate::base_connection::BaseConnection;
use crate::error::NetsshError;
use crate::vendors::cisco::{CiscoBaseConnection, CiscoDeviceConfig, CiscoDeviceConnection};
use async_trait::async_trait;
use log::{debug, warn};

pub struct CiscoNxosDevice {
    pub base: CiscoBaseConnection,
}

impl CiscoNxosDevice {
    pub fn new(config: CiscoDeviceConfig) -> Result<Self, NetsshError> {
        Ok(Self {
            base: CiscoBaseConnection::new(config)?,
        })
    }

    pub fn with_connection(connection: BaseConnection, config: CiscoDeviceConfig) -> Self {
        Self {
            base: CiscoBaseConnection::with_connection(connection, config),
        }
    }

    pub fn connect(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoNxosSsh::connect", "Connecting to NX-OS device");

        // Connect to the device using the base connection
        self.base.connection.connect(
            &self.base.config.host,
            &self.base.config.username,
            self.base.config.password.as_deref(),
            self.base.config.port,
            self.base.config.timeout,
        )?;

        if let Some(log_file) = &self.base.config.session_log {
            self.base.connection.set_session_log(log_file)?;
        }

        // Call our own session_preparation instead of the base class's
        self.session_preparation()?;

        Ok(())
    }

    pub fn check_enable_mode(&mut self) -> Result<bool, NetsshError> {
        debug!(target: "CiscoNxosSsh::check_enable_mode", "Delegating to CiscoBaseConnection::check_enable_mode");
        self.base.check_enable_mode()
    }

    pub fn enable(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoNxosSsh::enable", "Delegating to CiscoBaseConnection::enable");
        self.base.enable()
    }

    pub fn exit_enable_mode(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoNxosSsh::exit_enable_mode", "Delegating to CiscoBaseConnection::exit_enable_mode");
        self.base.exit_enable_mode()
    }

    pub fn close(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoNxosSsh::close", "Delegating to CiscoBaseConnection::close");
        self.base.close()
    }

    pub fn strip_ansi_escape_codes(&self, data: &str) -> String {
        self.base.strip_ansi_escape_codes(data)
    }
}

#[async_trait]
impl CiscoDeviceConnection for CiscoNxosDevice {
    fn session_preparation(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoNxosSsh::session_preparation", "Preparing NX-OS session");

        // Only open a channel if one doesn't already exist
        if self.base.connection.channel.is_none() {
            debug!(target: "CiscoNxosSsh::session_preparation", "Opening a new channel");
            self.base.connection.open_channel()?;
        } else {
            debug!(target: "CiscoNxosSsh::session_preparation", "Channel already exists, skipping open_channel");
        }

        // add delay to wait for the device to be ready
        // std::thread::sleep(std::time::Duration::from_millis(500));
        let output = self.base.connection.clear_buffer(Some("[>#]"), Some(20), None)?;
        debug!(target: "CiscoBaseConnection::session_preparation", "Cleared buffer: {}", output);

        debug!(target: "CiscoNxosSsh::session_preparation", "Setting base prompt");
        // Set base prompt
        self.set_base_prompt()?;

        // Configure terminal settings (calls our overridden terminal_settings method)
        self.terminal_settings()?;

        // Enter enable mode if not already in it
        if !self.check_enable_mode()? {
            debug!(target: "CiscoNxosSsh::session_preparation", "Not in privileged mode #, entering enable mode");
            self.base.enable()?;
        } else {
            debug!(target: "CiscoNxosSsh::session_preparation", "Already in privileged mode");
        }

        debug!(target: "CiscoNxosSsh::session_preparation", "Session preparation completed successfully");
        Ok(())
    }

    fn terminal_settings(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoNxosSsh::terminal_settings", "Configuring NX-OS terminal settings");

        // NX-OS specific terminal settings
        self.set_terminal_width(511)?;
        self.disable_paging()?;

        debug!(target: "CiscoNxosSsh::terminal_settings", "NX-OS terminal settings configured successfully");
        Ok(())
    }

    fn set_terminal_width(&mut self, width: u32) -> Result<(), NetsshError> {
        debug!(target: "CiscoNxosSsh::set_terminal_width", "Setting terminal width to {}", width);
        self.base.set_terminal_width(width)
    }

    fn disable_paging(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoNxosSsh::disable_paging", "Disabling paging for NX-OS");

        // NX-OS uses "terminal length 0" just like IOS
        self.base.disable_paging()
    }

    fn set_base_prompt(&mut self) -> Result<String, NetsshError> {
        debug!(target: "CiscoNxosSsh::set_base_prompt", "Setting base prompt for NX-OS");

        // Use the base implementation but set NX-OS-specific default prompt if needed
        let result = self.base.set_base_prompt();

        // If there was an error, set NX-OS-specific default prompt
        if result.is_err() {
            self.base.prompt = "NX-OS".to_string();
            self.base.connection.base_prompt = Some(self.base.prompt.clone());
            self.base
                .connection
                .channel
                .set_base_prompt(&self.base.prompt);
            return Ok(self.base.prompt.clone());
        }

        Ok(result?)
    }

    fn check_config_mode(&mut self) -> Result<bool, NetsshError> {
        debug!(target: "CiscoNxosSsh::check_config_mode", "Delegating to CiscoBaseConnection::check_config_mode");
        self.base.check_config_mode()
    }

    fn config_mode(&mut self, config_command: Option<&str>) -> Result<(), NetsshError> {
        debug!(target: "CiscoNxosSsh::config_mode", "Delegating to CiscoBaseConnection::config_mode");
        self.base.config_mode(config_command)
    }

    fn exit_config_mode(&mut self, exit_command: Option<&str>) -> Result<(), NetsshError> {
        debug!(target: "CiscoNxosSsh::exit_config_mode", "Delegating to CiscoBaseConnection::exit_config_mode");
        self.base.exit_config_mode(exit_command)
    }

    fn save_config(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoNxosSsh::save_config", "Saving configuration");

        // Ensure we're in enable mode
        if !self.check_enable_mode()? {
            debug!(target: "CiscoNxosSsh::save_config", "Not in enable mode, entering enable mode first");
            self.enable()?;
        }

        // Exit config mode if we're in it
        if self.check_config_mode()? {
            debug!(target: "CiscoNxosSsh::save_config", "In config mode, exiting config mode first");
            self.exit_config_mode(None)?;
        }

        // Send save command - default for IOS/NXOS
        self.base
            .connection
            .write_channel("copy running-config startup-config\n")?;

        // Wait for completion
        let output = self
            .base
            .connection
            .read_until_pattern(&self.base.prompt, None, None)?;

        debug!(target: "CiscoNxosSsh::save_config", "Save command output: {}", output);

        if output.contains("Error") {
            warn!(target: "CiscoNxosSsh::save_config", "Error saving configuration: {}", output);
            return Err(NetsshError::CommandError(format!(
                "Failed to save configuration: {}",
                output
            )));
        }

        debug!(target: "CiscoNxosSsh::save_config", "Configuration saved successfully");
        Ok(())
    }

    fn send_command(&mut self, command: &str) -> Result<String, NetsshError> {
        debug!(target: "CiscoNxosSsh::send_command", "Delegating to CiscoBaseConnection::send_command");
        self.base.send_command(command)
    }
}
