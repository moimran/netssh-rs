pub mod ios;
pub mod xr;
pub mod asa;

pub use ios::*;
pub use xr::CiscoXrSsh;
pub use asa::*;

use crate::error::NetmikoError;
use async_trait::async_trait;
use std::time::Duration;

#[async_trait]
pub trait CiscoBaseConnection {
    fn session_preparation(&mut self) -> Result<(), NetmikoError>;
    fn set_terminal_width(&mut self, width: u32) -> Result<(), NetmikoError>;
    fn disable_paging(&mut self) -> Result<(), NetmikoError>;
    fn set_base_prompt(&mut self) -> Result<String, NetmikoError>;
    fn check_config_mode(&mut self) -> Result<bool, NetmikoError>;
    fn config_mode(&mut self, config_command: Option<&str>) -> Result<(), NetmikoError>;
    fn exit_config_mode(&mut self, exit_command: Option<&str>) -> Result<(), NetmikoError>;
    fn save_config(&mut self) -> Result<(), NetmikoError>;
    fn send_command(&mut self, command: &str) -> Result<String, NetmikoError>;
    fn change_context(&mut self, _context_name: &str) -> Result<(), NetmikoError> {
        Ok(())
    }
}

pub trait CiscoBaseConnectionTrait {
    fn session_preparation(&mut self) -> Result<(), crate::error::NetmikoError>;
    fn set_terminal_width(&mut self, width: u32) -> Result<(), crate::error::NetmikoError>;
    fn disable_paging(&mut self) -> Result<(), crate::error::NetmikoError>;
    fn set_base_prompt(&mut self) -> Result<String, crate::error::NetmikoError>;
    fn check_config_mode(&mut self) -> Result<bool, crate::error::NetmikoError>;
    fn config_mode(&mut self, config_command: Option<&str>) -> Result<(), crate::error::NetmikoError>;
    fn exit_config_mode(&mut self, exit_command: Option<&str>) -> Result<(), crate::error::NetmikoError>;
    fn save_config(&mut self) -> Result<(), crate::error::NetmikoError>;
    fn send_command(&mut self, command: &str) -> Result<String, crate::error::NetmikoError>;
}

#[derive(Debug, Clone)]
pub struct CiscoDeviceConfig {
    pub host: String,
    pub username: String,
    pub password: Option<String>,
    pub port: Option<u16>,
    pub timeout: Option<Duration>,
    pub secret: Option<String>,
    pub session_log: Option<String>,
}

impl Default for CiscoDeviceConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            username: String::new(),
            password: None,
            port: None,
            timeout: None,
            secret: None,
            session_log: None,
        }
    }
}
