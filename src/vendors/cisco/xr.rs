use crate::{BaseConnection, NetmikoError};
use crate::vendors::cisco::CiscoBaseConnection;
use log::debug;
use std::time::Duration;

pub struct CiscoXrSsh {
    base: BaseConnection,
}

impl CiscoBaseConnection for CiscoXrSsh {
    fn session_preparation(&mut self) -> Result<(), NetmikoError> {
        debug!("Preparing XR session");
        self.disable_paging()?;
        self.set_terminal_width(511)?;
        Ok(())
    }

    fn set_terminal_width(&mut self, width: u32) -> Result<(), NetmikoError> {
        debug!("Setting terminal width to {}", width);
        self.base.send_command(&format!("terminal width {}", width))?;
        Ok(())
    }

    fn disable_paging(&mut self) -> Result<(), NetmikoError> {
        debug!("Disabling paging");
        self.base.send_command("terminal length 0")?;
        Ok(())
    }

    fn set_base_prompt(&mut self) -> Result<String, NetmikoError> {
        debug!("Setting base prompt for XR device");
        let output = self.base.send_command("\n")?;
        if let Some(prompt) = output.lines().last() {
            if prompt.ends_with('#') {
                let prompt = prompt.trim_end_matches('#').to_string();
                Ok(prompt)
            } else {
                Err(NetmikoError::PromptError("Failed to find XR prompt ending with #".to_string()))
            }
        } else {
            Err(NetmikoError::PromptError("No output received when finding prompt".to_string()))
        }
    }

    fn check_config_mode(&mut self) -> Result<bool, NetmikoError> {
        debug!("Checking config mode");
        let prompt = self.base.send_command("\n")?;
        Ok(prompt.contains("(config)"))
    }

    fn config_mode(&mut self, config_command: Option<&str>) -> Result<(), NetmikoError> {
        debug!("Entering configuration mode");
        let cmd = config_command.unwrap_or("configure terminal");
        self.base.send_command(cmd)?;
        if !self.check_config_mode()? {
            Err(NetmikoError::ConfigError("Failed to enter configuration mode".to_string()))
        } else {
            Ok(())
        }
    }

    fn exit_config_mode(&mut self, exit_command: Option<&str>) -> Result<(), NetmikoError> {
        debug!("Exiting configuration mode");
        let cmd = exit_command.unwrap_or("end");
        self.base.send_command(cmd)?;
        if self.check_config_mode()? {
            Err(NetmikoError::ConfigError("Failed to exit configuration mode".to_string()))
        } else {
            Ok(())
        }
    }

    fn save_config(&mut self) -> Result<(), NetmikoError> {
        debug!("Saving configuration");
        self.base.send_command("commit")?;
        Ok(())
    }

    fn send_command(&mut self, command: &str) -> Result<String, NetmikoError> {
        debug!("Sending command: {}", command);
        self.base.send_command(command)
    }
}

impl CiscoXrSsh {
    pub fn new() -> Result<Self, NetmikoError> {
        debug!("Creating new CiscoXrSsh instance");
        Ok(CiscoXrSsh {
            base: BaseConnection::new()?,
        })
    }

    pub fn establish_connection(
        &mut self,
        host: &str,
        username: &str,
        password: Option<&str>,
        port: Option<u16>,
        timeout: Option<Duration>,
    ) -> Result<(), NetmikoError> {
        debug!("Establishing connection to Cisco XR device");
        self.base.connect(host, username, password, port, timeout)?;
        let prompt = self.set_base_prompt()?;
        self.session_preparation()?;
        debug!("Successfully connected to device with prompt: {}", prompt);
        Ok(())
    }

    pub fn disconnect(&mut self) -> Result<(), NetmikoError> {
        debug!("Disconnecting from device");
        if let Some(session) = self.base.session.take() {
            session.disconnect(None, "", None)
                .map_err(|e| NetmikoError::DisconnectError(e.to_string()))?;
        }
        Ok(())
    }
}
