use crate::base_connection::BaseConnection;
use crate::error::NetsshError;
use crate::vendors::cisco::{CiscoBaseConnection, CiscoDeviceConfig, CiscoDeviceConnection};
use crate::vendors::common::DefaultConfigSetMethods;
use async_trait::async_trait;
use tracing::{debug, instrument, warn};

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
        debug!(target: "CiscoNxosDevice::connect", "Connecting to NX-OS device");

        // Convert String to &str through Option by using as_str()
        let host = self.base.config.host.as_str();
        let username = self.base.config.username.as_str();

        // Connect to the device using the base connection
        self.base.connection.connect(
            Some(host),
            Some(username),
            self.base.config.password.as_deref(),
            self.base.config.port,
            self.base.config.timeout,
        )?;

        if let Some(log_file) = &self.base.config.session_log {
            self.base.connection.set_session_log(log_file)?;
        }
        self.session_preparation()?;

        Ok(())
    }

    pub fn check_enable_mode(&mut self) -> Result<bool, NetsshError> {
        debug!(target: "CiscoNxosDevice::check_enable_mode", "Delegating to CiscoBaseConnection::check_enable_mode");
        self.base.check_enable_mode()
    }

    pub fn enable(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoNxosDevice::enable", "Delegating to CiscoBaseConnection::enable");
        self.base.enable()
    }

    pub fn exit_enable_mode(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoNxosDevice::exit_enable_mode", "Delegating to CiscoBaseConnection::exit_enable_mode");
        self.base.exit_enable_mode()
    }

    pub fn close(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoNxosDevice::close", "Delegating to CiscoBaseConnection::close");
        self.base.close()
    }

    pub fn strip_ansi_escape_codes(&self, data: &str) -> String {
        self.base.strip_ansi_escape_codes(data)
    }

    // Use this save_config_with_params as an implementation detail
    // #[tracing::instrument(name = "CiscoNxosDevice::save_config", skip(self), level = "debug")]
    pub fn save_config_with_params(
        &mut self,
        cmd: &str,
        confirm: bool,
        confirm_response: Option<&str>,
    ) -> Result<String, NetsshError> {
        debug!(target: "CiscoNxosDevice::save_config", "Saving configuration with cmd: {}, confirm: {}", cmd, confirm);

        // Ensure we're in enable mode
        if !self.check_enable_mode()? {
            debug!(target: "CiscoNxosDevice::save_config", "Not in enable mode, entering enable mode first");
            self.enable()?;
        }

        let mut output = String::new();

        if confirm {
            // Use send_command_timing for the initial command when confirmation is needed
            let cmd_output = self.base.connection.send_command_timing(
                cmd,
                None,        // last_read
                None,        // read_timeout
                Some(false), // strip_prompt
                Some(false), // strip_command
                Some(true),  // normalize
                None,        // cmd_verify
            )?;
            output.push_str(&cmd_output);

            // Handle confirmation prompt
            if let Some(response) = confirm_response {
                let conf_output = self.base.connection.send_command_timing(
                    response,
                    None,        // last_read
                    None,        // read_timeout
                    Some(false), // strip_prompt
                    Some(false), // strip_command
                    Some(true),  // normalize
                    None,        // cmd_verify
                )?;
                output.push_str(&conf_output);
            } else {
                // Send enter by default
                let conf_output = self.base.connection.send_command_timing(
                    "\n",
                    None,        // last_read
                    None,        // read_timeout
                    Some(false), // strip_prompt
                    Some(false), // strip_command
                    Some(true),  // normalize
                    None,        // cmd_verify
                )?;
                output.push_str(&conf_output);
            }
        } else {
            // NX-OS is very slow on save_config, ensure it waits long enough
            debug!(target: "CiscoNxosDevice::save_config", "Using long timeout for NX-OS save");
            let cmd_output = self.base.connection.send_command(
                cmd,
                None,        // expect_string
                Some(100.0), // read_timeout - NX-OS needs long timeout
                Some(true),  // auto_find_prompt
                Some(false), // strip_prompt
                Some(false), // strip_command
                Some(true),  // normalize
                Some(true),  // cmd_verify
            )?;
            output.push_str(&cmd_output);
        }

        // Check for errors in the output
        if output.contains("Error") {
            warn!(target: "CiscoNxosDevice::save_config", "Error saving configuration: {}", output);
            return Err(NetsshError::CommandError(format!(
                "Failed to save configuration: {}",
                output
            )));
        }

        debug!(target: "CiscoNxosDevice::save_config", "Configuration saved successfully");
        Ok(output)
    }

    // Custom method for setting terminal width with pattern
    fn set_terminal_width_with_pattern(
        &mut self,
        width: u32,
        pattern: &str,
    ) -> Result<(), NetsshError> {
        debug!(target: "CiscoNxosDevice::set_terminal_width_with_pattern", 
               "Setting NX-OS terminal width to {} with pattern {}", width, pattern);

        let cmd = format!("terminal width {}\n", width);
        self.base.connection.write_channel(&cmd)?;

        // Read until the pattern is found
        let output = self
            .base
            .connection
            .read_until_pattern(pattern, None, None)?;
        debug!(target: "CiscoNxosDevice::set_terminal_width_with_pattern", 
               "Terminal width response: {}", output);

        Ok(())
    }

    // NX-OS specific normalization of line feeds (handle the extra carriage returns)
    pub fn normalize_linefeeds(&self, data: &str) -> String {
        // Implement the NX-OS specific line feed normalization logic
        // Convert '\r\n' or '\r\r\n' to '\n, and remove extra '\r's in the text
        let result = data
            .replace("\r\r\n\r", "\n")
            .replace("\r\r\n", "\n")
            .replace("\r\n", "\n")
            .replace("\r", "\n");

        debug!(target: "CiscoNxosDevice::normalize_linefeeds", "Normalized line feeds");
        result
    }
}

impl DefaultConfigSetMethods for CiscoNxosDevice {
    fn get_base_connection(&mut self) -> &mut BaseConnection {
        &mut self.base.connection
    }
}

#[async_trait]
impl CiscoDeviceConnection for CiscoNxosDevice {
    fn session_preparation(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoNxosDevice::session_preparation", "Preparing NX-OS session");

        // Enable ANSI escape code handling for NX-OS (matches Python's self.ansi_escape_codes = True)
        self.base.connection.ansi_escape_codes = true;

        // NX-OS has an issue where it echoes the command even though it hasn't returned the prompt
        self.base
            .connection
            .test_channel_read(None, Some(r"[>#]"))?;

        // Set terminal width with specific pattern for NX-OS
        self.set_terminal_width_with_pattern(511, "terminal width 511")?;

        // Disable paging
        self.disable_paging()?;

        // Set base prompt
        self.set_base_prompt()?;

        // Enter enable mode if needed
        if !self.check_enable_mode()? {
            debug!(target: "CiscoNxosDevice::session_preparation", "Not in privileged mode, entering enable mode");
            self.enable()?;
        }

        debug!(target: "CiscoNxosDevice::session_preparation", "NX-OS session preparation completed successfully");
        Ok(())
    }

    fn terminal_settings(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoNxosDevice::terminal_settings", "Configuring NX-OS terminal settings");

        // NX-OS specific terminal settings
        self.set_terminal_width(511)?;
        self.disable_paging()?;

        debug!(target: "CiscoNxosDevice::terminal_settings", "NX-OS terminal settings configured successfully");
        Ok(())
    }

    fn set_terminal_width(&mut self, width: u32) -> Result<(), NetsshError> {
        debug!(target: "CiscoNxosDevice::set_terminal_width", "Setting terminal width to {}", width);
        self.base.set_terminal_width(width)
    }

    fn disable_paging(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoNxosDevice::disable_paging", "Disabling paging for NX-OS");

        // NX-OS uses "terminal length 0" just like IOS
        self.base.disable_paging()
    }

    fn set_base_prompt(&mut self) -> Result<String, NetsshError> {
        debug!(target: "CiscoNxosDevice::set_base_prompt", "Setting base prompt for NX-OS");

        // Call the base implementation with NX-OS specific defaults
        self.base.connection.set_base_prompt(
            Some("#"), // Primary prompt terminator
            Some(">"), // Alternate prompt terminator
            None,      // Use default delay factor
            None,      // Use default pattern
        )
    }

    fn check_config_mode(&mut self) -> Result<bool, NetsshError> {
        debug!(target: "CiscoNxosDevice::check_config_mode", "Delegating to CiscoBaseConnection::check_config_mode");
        self.base.check_config_mode()
    }

    fn config_mode(&mut self, config_command: Option<&str>) -> Result<(), NetsshError> {
        debug!(target: "CiscoNxosDevice::config_mode", "Delegating to CiscoBaseConnection::config_mode");
        self.base.config_mode(config_command)
    }

    fn exit_config_mode(&mut self, exit_command: Option<&str>) -> Result<(), NetsshError> {
        debug!(target: "CiscoNxosDevice::exit_config_mode", "Delegating to CiscoBaseConnection::exit_config_mode");
        self.base.exit_config_mode(exit_command)
    }

    // #[instrument(name = "CiscoNxosDevice::send_command", skip(self), level = "debug")]
    fn send_command(&mut self, command: &str) -> Result<String, NetsshError> {
        debug!(target: "CiscoNxosDevice::send_command", "Delegating to CiscoBaseConnection::send_command");
        self.base.send_command(command)
    }

    fn save_config(&mut self) -> Result<String, NetsshError> {
        // Use the default command specific to NX-OS with NX-OS appropriate parameters
        self.save_config_with_params("copy running-config startup-config", false, None)
    }
}
