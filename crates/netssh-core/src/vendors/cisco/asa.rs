use crate::base_connection::BaseConnection;
use crate::error::NetsshError;
use crate::vendors::cisco::{CiscoBaseConnection, CiscoDeviceConfig, CiscoDeviceConnection};
use crate::vendors::common::DefaultConfigSetMethods;
use async_trait::async_trait;
use tracing::debug;

pub struct CiscoAsaDevice {
    pub base: CiscoBaseConnection,
    context: Option<String>,
}

impl CiscoAsaDevice {
    pub fn new(config: CiscoDeviceConfig) -> Result<Self, NetsshError> {
        Ok(Self {
            base: CiscoBaseConnection::new(config)?,
            context: None,
        })
    }

    pub fn with_connection(connection: BaseConnection, config: CiscoDeviceConfig) -> Self {
        Self {
            base: CiscoBaseConnection::with_connection(connection, config),
            context: None,
        }
    }

    pub fn connect(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoAsaDevice::connect", "Connecting to ASA device");

        // Connect to the device using the base connection
        self.base.connection.connect(
            Some(&self.base.config.host),
            Some(&self.base.config.username),
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
        debug!(target: "CiscoAsaDevice::check_enable_mode", "Delegating to CiscoBaseConnection::check_enable_mode");
        self.base.check_enable_mode()
    }

    pub fn enable(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoAsaDevice::enable", "Delegating to CiscoBaseConnection::enable");
        let result = self.base.enable();

        // ASA-specific: Call session_preparation after enable
        if result.is_ok() {
            self.terminal_settings()?;
        }

        result
    }

    pub fn exit_enable_mode(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoAsaDevice::exit_enable_mode", "Delegating to CiscoBaseConnection::exit_enable_mode");
        self.base.exit_enable_mode()
    }

    pub fn close(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoAsaDevice::close", "Delegating to CiscoBaseConnection::close");
        self.base.close()
    }

    pub fn strip_ansi_escape_codes(&self, data: &str) -> String {
        self.base.strip_ansi_escape_codes(data)
    }

    // ASA-specific methods
    pub fn change_context(&mut self, context_name: &str) -> Result<(), NetsshError> {
        debug!(target: "CiscoAsaDevice::change_context", "Changing to context: {}", context_name);

        // Send the changeto context command
        self.base
            .connection
            .write_channel(&format!("changeto context {}\n", context_name))?;

        // Wait for prompt
        let output = self
            .base
            .connection
            .read_until_pattern(r"[>#]", None, None)?;

        // Check if the command was successful
        if output.contains("ERROR") || output.contains("Invalid") {
            debug!(target: "CiscoAsaDevice::change_context", "Error changing context: {}", output);
            return Err(NetsshError::CommandError(format!(
                "Failed to change to context {}: {}",
                context_name, output
            )));
        }

        // Update the context
        self.context = Some(context_name.to_string());

        // Update the prompt after context change
        self.set_base_prompt()?;

        debug!(target: "CiscoAsaDevice::change_context", "Successfully changed to context: {}", context_name);
        Ok(())
    }

    /// Configure terminal settings - can be overridden by device-specific implementations
    pub fn terminal_settings(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoAsaDevice::terminal_settings", "Configuring base terminal settings");

        // Disable paging
        self.disable_paging()?;

        debug!(target: "CiscoBaseConnection::terminal_settings", "Base terminal settings configured successfully");
        Ok(())
    }

    pub fn get_current_context(&self) -> Option<&str> {
        self.context.as_deref()
    }
}

impl DefaultConfigSetMethods for CiscoAsaDevice {
    fn get_base_connection(&mut self) -> &mut BaseConnection {
        &mut self.base.connection
    }
}

#[async_trait]
impl CiscoDeviceConnection for CiscoAsaDevice {
    fn session_preparation(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoAsaDevice::session_preparation", "Preparing ASA session");

        // Only open a channel if one doesn't already exist
        if self.base.connection.channel.is_none() {
            debug!(target: "CiscoAsaDevice::session_preparation", "Opening a new channel");
            self.base.connection.open_channel()?;
        } else {
            debug!(target: "CiscoAsaDevice::session_preparation", "Channel already exists, skipping open_channel");
        }

        // add delay to wait for the device to be ready
        std::thread::sleep(std::time::Duration::from_millis(500));

        debug!(target: "CiscoAsaDevice::session_preparation", "Setting base prompt");
        // Set base prompt
        self.set_base_prompt()?;

        // Enter enable mode if not already in it
        if !self.check_enable_mode()? {
            debug!(target: "CiscoAsaDevice::session_preparation", "Not in privileged mode #, entering enable mode");
            self.enable()?;
        } else {
            debug!(target: "CiscoAsaDevice::session_preparation", "Already in privileged mode");
        }

        debug!(target: "CiscoAsaDevice::session_preparation", "Session preparation completed successfully");
        Ok(())
    }

    fn terminal_settings(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoAsaDevice::terminal_settings", "Configuring ASA terminal settings");

        // ASA-specific terminal settings - no terminal width, just disable paging
        self.disable_paging()?;

        debug!(target: "CiscoAsaDevice::terminal_settings", "ASA terminal settings configured successfully");
        Ok(())
    }

    fn set_terminal_width(&mut self, width: u32) -> Result<(), NetsshError> {
        debug!(target: "CiscoAsaDevice::set_terminal_width", "Delegating to CiscoBaseConnection::set_terminal_width");
        self.base.set_terminal_width(width)
    }

    fn disable_paging(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoAsaDevice::disable_paging", "Disabling paging (ASA-specific)");

        // Send the command with a newline - ASA uses "terminal pager 0"
        self.base.connection.write_channel("terminal pager 0\n")?;

        // Wait for a short time to ensure the command is processed
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Read whatever is available
        let output = match self.base.connection.read_channel() {
            Ok(out) => out,
            Err(e) => {
                debug!(target: "CiscoAsaDevice::disable_paging", "Error reading response: {}", e);
                // Continue anyway, don't fail the connection for this
                String::new()
            }
        };

        if output.contains("Invalid") || output.contains("Error") {
            debug!(target: "CiscoAsaDevice::disable_paging", "Error disabling paging: {}", output);
            // Continue anyway, don't fail the connection for this
        } else {
            debug!(target: "CiscoAsaDevice::disable_paging", "Paging disabled successfully");
        }

        // Always return success, even if there was an error
        Ok(())
    }

    fn set_base_prompt(&mut self) -> Result<String, NetsshError> {
        debug!(target: "CiscoAsaDevice::set_base_prompt", "Setting base prompt for ASA");

        // Use the base implementation but set ASA-specific default prompt if needed
        let result = self.base.set_base_prompt();

        // If there was an error, set ASA-specific default prompt
        if result.is_err() {
            self.base.prompt = "ASA".to_string();
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
        debug!(target: "CiscoAsaDevice::check_config_mode", "Delegating to CiscoBaseConnection::check_config_mode");
        self.base.check_config_mode()
    }

    fn config_mode(&mut self, config_command: Option<&str>) -> Result<(), NetsshError> {
        debug!(target: "CiscoAsaDevice::config_mode", "Delegating to CiscoBaseConnection::config_mode");
        self.base.config_mode(config_command)
    }

    fn exit_config_mode(&mut self, exit_command: Option<&str>) -> Result<(), NetsshError> {
        debug!(target: "CiscoAsaDevice::exit_config_mode", "Delegating to CiscoBaseConnection::exit_config_mode");
        self.base.exit_config_mode(exit_command)
    }

    fn save_config(&mut self) -> Result<String, NetsshError> {
        debug!(target: "CiscoAsaDevice::save_config", "Saving ASA configuration");

        // Ensure we're in enable mode
        if !self.check_enable_mode()? {
            debug!(target: "CiscoAsaDevice::save_config", "Not in enable mode, entering enable mode first");
            self.enable()?;
        }

        // Exit config mode if we're in it
        if self.check_config_mode()? {
            debug!(target: "CiscoAsaDevice::save_config", "In config mode, exiting config mode first");
            self.exit_config_mode(None)?;
        }

        // Send save command - ASA uses "write memory"
        self.base.connection.write_channel("write memory\n")?;

        // Wait for completion
        let output = self.base.connection.read_until_pattern("#", None, None)?;

        if output.contains("Error") {
            debug!(target: "CiscoAsaDevice::save_config", "Error saving configuration: {}", output);
            return Err(NetsshError::CommandError(format!(
                "Failed to save configuration: {}",
                output
            )));
        }

        debug!(target: "CiscoAsaDevice::save_config", "Configuration saved successfully");
        Ok(output)
    }

    fn send_command(&mut self, command: &str) -> Result<String, NetsshError> {
        debug!(target: "CiscoAsaDevice::send_command", "Delegating to CiscoBaseConnection::send_command");
        self.base.send_command(command)
    }

    fn change_context(&mut self, context_name: &str) -> Result<(), NetsshError> {
        // This method is already implemented as a public method in the CiscoAsaDevice impl block
        self.change_context(context_name)
    }
}
