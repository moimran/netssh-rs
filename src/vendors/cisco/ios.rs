use crate::base_connection::BaseConnection;
use crate::error::NetsshError;
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
    pub fn close(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoIosDevice::close", "Closing connection to device");
        
        // Close the channel if it exists
        if let Some(channel) = self.connection.channel.as_mut() {
            debug!(target: "CiscoIosDevice::close", "Closing SSH channel");
            channel.close().map_err(|e| NetsshError::SshError(e))?;
            channel.wait_close().map_err(|e| NetsshError::SshError(e))?;
        }
        
        // Clear the channel reference
        self.connection.channel = None;
        
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

        if let Some(log_file) = &self.config.session_log {
            self.connection.set_session_log(log_file)?;
        }

        self.session_preparation()?;
        Ok(())
    }

    fn enable(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoIosDevice::enable", "Entering enable mode");
        
        // Send enable command (short form)
        self.connection.write_channel("en\n")?;
        
        // Wait for password prompt if secret is provided
        if let Some(secret) = &self.config.secret {
            debug!(target: "CiscoIosDevice::enable", "Waiting for password prompt");
            // Use a more flexible pattern to match "Password:" with possible whitespace
            self.connection.read_until_pattern("[Pp]assword\\s*:")?;
            
            debug!(target: "CiscoIosDevice::enable", "Sending enable password");
            self.connection.write_channel(&format!("{}\n", secret))?;
        }
        
        // Wait for enable prompt (the # character)
        self.connection.read_until_pattern("#")?;
        debug!(target: "CiscoIosDevice::enable", "Successfully entered enable mode");
        
        Ok(())
    }
}

#[async_trait]
impl CiscoBaseConnection for CiscoIosDevice {
    fn session_preparation(&mut self) -> Result<(), NetsshError> {
        // Only open a channel if one doesn't already exist
        if self.connection.channel.is_none() {
            debug!(target: "CiscoIosDevice::session_preparation", "Opening a new channel");
            self.connection.open_channel()?;
        } else {
            debug!(target: "CiscoIosDevice::session_preparation", "Channel already exists, skipping open_channel");
        }
        
        self.set_terminal_width(511)?;
        self.disable_paging()?;
        self.set_base_prompt()?;
        
        if !self.check_config_mode()? {
            self.enable()?;
        }
        
        Ok(())
    }

    fn set_terminal_width(&mut self, width: u32) -> Result<(), NetsshError> {
        debug!(target: "CiscoIosDevice::set_terminal_width", "Setting terminal width to {}", width);
        self.connection.write_channel(&format!("terminal width {}\n", width))?;
        self.connection.read_until_pattern("#")?;
        Ok(())
    }

    fn disable_paging(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoIosDevice::disable_paging", "Disabling paging");
        self.connection.write_channel("terminal length 0\n")?;
        self.connection.read_until_pattern("#")?;
        Ok(())
    }

    fn set_base_prompt(&mut self) -> Result<String, NetsshError> {
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

    fn check_config_mode(&mut self) -> Result<bool, NetsshError> {
        self.connection.write_channel("\n")?;
        let output = self.connection.read_until_pattern("[>#]")?;
        Ok(CONFIG_PATTERN.is_match(&output))
    }

    fn config_mode(&mut self, config_command: Option<&str>) -> Result<(), NetsshError> {
        let cmd = config_command.unwrap_or("configure terminal");
        self.connection.write_channel(&format!("{}\n", cmd))?;
        self.connection.read_until_pattern("\\(config\\)#")?;
        Ok(())
    }

    fn exit_config_mode(&mut self, exit_command: Option<&str>) -> Result<(), NetsshError> {
        let cmd = exit_command.unwrap_or("end");
        self.connection.write_channel(&format!("{}\n", cmd))?;
        self.connection.read_until_pattern("#")?;
        Ok(())
    }

    fn save_config(&mut self) -> Result<(), NetsshError> {
        self.connection.write_channel("write memory\n")?;
        self.connection.read_until_pattern("#")?;
        Ok(())
    }

    fn send_command(&mut self, command: &str) -> Result<String, NetsshError> {
        debug!(target: "CiscoIosDevice::send_command", "Sending command: {}", command);
        self.connection.send_command(command)
    }
}
