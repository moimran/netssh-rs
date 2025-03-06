use crate::base_connection::BaseConnection;
use crate::error::NetsshError;
use crate::vendors::cisco::{CiscoBaseConnection, CiscoDeviceConfig};
use async_trait::async_trait;

pub struct CiscoAsaDevice {
    connection: BaseConnection,
    config: CiscoDeviceConfig,
    prompt: String,
    context: Option<String>,
}

impl CiscoAsaDevice {
    pub fn new(config: CiscoDeviceConfig) -> Result<Self, NetsshError> {
        Ok(Self {
            connection: BaseConnection::new()?,
            config,
            prompt: String::new(),
            context: None,
        })
    }

    pub fn connect(&mut self) -> Result<(), NetsshError> {
        self.connection.connect(
            &self.config.host,
            &self.config.username,
            self.config.password.as_deref(),
            self.config.port,
            self.config.timeout,
        )?;
        
        self.session_preparation()?;
        Ok(())
    }

    pub fn enable(&mut self, secret: &str) -> Result<(), NetsshError> {
        // Check if we're already in enable mode
        if self.check_enable_mode()? {
            return Ok(());
        }

        // Send enable command and secret
        self.connection.write_channel("enable\n")?;
        self.connection.write_channel(&format!("{}\n", secret))?;

        // Check if we entered enable mode
        if !self.check_enable_mode()? {
            return Err(NetsshError::AuthenticationError(
                "Failed to enter enable mode".to_string(),
            ));
        }

        Ok(())
    }

    fn check_enable_mode(&mut self) -> Result<bool, NetsshError> {
        self.connection.write_channel("\n")?;
        let output = self.connection.read_until_pattern("#")?;
        Ok(output.contains("#"))
    }
}

#[async_trait]
impl CiscoBaseConnection for CiscoAsaDevice {
    fn session_preparation(&mut self) -> Result<(), NetsshError> {
        // Disable paging
        self.disable_paging()?;

        // Set terminal width
        self.set_terminal_width(511)?;

        // Set base prompt
        self.prompt = self.set_base_prompt()?;

        Ok(())
    }

    fn set_terminal_width(&mut self, width: u32) -> Result<(), NetsshError> {
        self.connection.write_channel(&format!("terminal width {}\n", width))?;
        Ok(())
    }

    fn disable_paging(&mut self) -> Result<(), NetsshError> {
        self.connection.write_channel("terminal pager 0\n")?;
        Ok(())
    }

    fn set_base_prompt(&mut self) -> Result<String, NetsshError> {
        self.connection.write_channel("\n")?;
        let output = self.connection.read_until_pattern(r"[>#]")?;
        
        // Extract prompt from output
        let prompt = output.lines()
            .last()
            .ok_or_else(|| NetsshError::CommandError("Failed to get prompt".to_string()))?
            .trim()
            .trim_end_matches(&['>', '#'][..])
            .to_string();

        Ok(prompt)
    }

    fn check_config_mode(&mut self) -> Result<bool, NetsshError> {
        self.connection.write_channel("\n")?;
        let output = self.connection.read_until_pattern(r"[>#]")?;
        Ok(output.contains("(config)#"))
    }

    fn config_mode(&mut self, config_command: Option<&str>) -> Result<(), NetsshError> {
        let cmd = config_command.unwrap_or("configure terminal");
        self.connection.write_channel(&format!("{}\n", cmd))?;
        
        if !self.check_config_mode()? {
            return Err(NetsshError::CommandError("Failed to enter config mode".to_string()));
        }
        Ok(())
    }

    fn exit_config_mode(&mut self, exit_command: Option<&str>) -> Result<(), NetsshError> {
        let cmd = exit_command.unwrap_or("end");
        self.connection.write_channel(&format!("{}\n", cmd))?;
        
        if self.check_config_mode()? {
            return Err(NetsshError::CommandError("Failed to exit config mode".to_string()));
        }
        Ok(())
    }

    fn save_config(&mut self) -> Result<(), NetsshError> {
        self.connection.write_channel("write memory\n")?;
        Ok(())
    }

    fn send_command(&mut self, command: &str) -> Result<String, NetsshError> {
        self.connection.write_channel(&format!("{}\n", command))?;
        self.connection.read_until_pattern(&self.prompt)
    }

    fn change_context(&mut self, context_name: &str) -> Result<(), NetsshError> {
        self.connection.write_channel(&format!("changeto context {}\n", context_name))?;
        self.context = Some(context_name.to_string());
        Ok(())
    }
}
