use crate::base_connection::BaseConnection;
use crate::error::NetmikoError;
use crate::vendors::cisco::{CiscoBaseConnection, CiscoDeviceConfig};
use async_trait::async_trait;
use regex::Regex;
use lazy_static::lazy_static;
use log::debug;

lazy_static! {
    static ref PROMPT_PATTERN: Regex = Regex::new(r"[>#]").unwrap();
    static ref CONFIG_PATTERN: Regex = Regex::new(r"\)#").unwrap();
}

pub struct CiscoIosDevice {
    connection: BaseConnection,
    config: CiscoDeviceConfig,
    prompt: String,
}

impl CiscoIosDevice {
    pub fn new(config: CiscoDeviceConfig) -> Result<Self, NetmikoError> {
        Ok(Self {
            connection: BaseConnection::new()?,
            config,
            prompt: String::new(),
        })
    }

    pub fn connect(&mut self) -> Result<(), NetmikoError> {
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

    fn enable(&mut self) -> Result<(), NetmikoError> {
        debug!(target: "CiscoIosDevice::enable", "Entering enable mode");
        
        // Send enable command
        self.connection.write_channel("enable\n")?;
        
        // Wait for password prompt if secret is provided
        if let Some(secret) = &self.config.secret {
            debug!(target: "CiscoIosDevice::enable", "Waiting for password prompt");
            self.connection.read_until_pattern("Password:")?;
            
            debug!(target: "CiscoIosDevice::enable", "Sending enable password");
            self.connection.write_channel(&format!("{}\n", secret))?;
        }
        
        // Wait for enable prompt
        self.connection.read_until_pattern("#")?;
        debug!(target: "CiscoIosDevice::enable", "Successfully entered enable mode");
        
        Ok(())
    }
}

#[async_trait]
impl CiscoBaseConnection for CiscoIosDevice {
    fn session_preparation(&mut self) -> Result<(), NetmikoError> {
        self.connection.open_channel()?;
        self.set_terminal_width(511)?;
        self.disable_paging()?;
        self.set_base_prompt()?;
        
        if !self.check_config_mode()? {
            self.enable()?;
        }
        
        Ok(())
    }

    fn set_terminal_width(&mut self, width: u32) -> Result<(), NetmikoError> {
        debug!(target: "CiscoIosDevice::set_terminal_width", "Setting terminal width to {}", width);
        self.connection.write_channel(&format!("terminal width {}\n", width))?;
        self.connection.read_until_pattern("#")?;
        Ok(())
    }

    fn disable_paging(&mut self) -> Result<(), NetmikoError> {
        debug!(target: "CiscoIosDevice::disable_paging", "Disabling paging");
        self.connection.write_channel("terminal length 0\n")?;
        self.connection.read_until_pattern("#")?;
        Ok(())
    }

    fn set_base_prompt(&mut self) -> Result<String, NetmikoError> {
        debug!(target: "CiscoIosDevice::set_base_prompt", "Setting base prompt");
        self.connection.write_channel("\n")?;
        let output = self.connection.read_until_pattern("[>#]")?;
        
        if let Some(last_line) = output.lines().last() {
            if let Some(prompt) = PROMPT_PATTERN.find(last_line) {
                self.prompt = last_line[..prompt.start()].trim_end().to_string();
                if self.prompt.len() > 16 {
                    self.prompt = self.prompt[..16].to_string();
                }
            }
        }
        
        Ok(self.prompt.clone())
    }

    fn check_config_mode(&mut self) -> Result<bool, NetmikoError> {
        self.connection.write_channel("\n")?;
        let output = self.connection.read_until_pattern("[>#]")?;
        Ok(CONFIG_PATTERN.is_match(&output))
    }

    fn config_mode(&mut self, config_command: Option<&str>) -> Result<(), NetmikoError> {
        let cmd = config_command.unwrap_or("configure terminal");
        self.connection.write_channel(&format!("{}\n", cmd))?;
        self.connection.read_until_pattern("\\(config\\)#")?;
        Ok(())
    }

    fn exit_config_mode(&mut self, exit_command: Option<&str>) -> Result<(), NetmikoError> {
        let cmd = exit_command.unwrap_or("end");
        self.connection.write_channel(&format!("{}\n", cmd))?;
        self.connection.read_until_pattern("#")?;
        Ok(())
    }

    fn save_config(&mut self) -> Result<(), NetmikoError> {
        self.connection.write_channel("write memory\n")?;
        self.connection.read_until_pattern("#")?;
        Ok(())
    }

    fn send_command(&mut self, command: &str) -> Result<String, NetmikoError> {
        debug!(target: "CiscoIosDevice::send_command", "Sending command: {}", command);
        self.connection.send_command(command)
    }
}
