pub mod asa;
pub mod asa_network_device;
pub mod ciscodevicebase;
pub mod ios;
pub mod ios_network_device;
pub mod nxos;
pub mod nxos_network_device;
pub mod xr;
pub mod xr_network_device;

pub use asa::CiscoAsaDevice;
pub use ciscodevicebase::CiscoBaseConnection;
pub use ios::CiscoIosDevice;
pub use nxos::CiscoNxosDevice;
pub use xr::CiscoXrDevice;

use crate::error::NetsshError;
use crate::vendors::common::DefaultConfigSetMethods;
use async_trait::async_trait;
use std::time::Duration;

#[async_trait]
pub trait CiscoDeviceConnection: DefaultConfigSetMethods {
    fn session_preparation(&mut self) -> Result<(), NetsshError>;
    fn terminal_settings(&mut self) -> Result<(), NetsshError>;
    fn set_terminal_width(&mut self, width: u32) -> Result<(), NetsshError>;
    fn disable_paging(&mut self) -> Result<(), NetsshError>;
    fn set_base_prompt(&mut self) -> Result<String, NetsshError>;
    fn check_config_mode(&mut self) -> Result<bool, NetsshError>;
    fn config_mode(&mut self, config_command: Option<&str>) -> Result<(), NetsshError>;
    fn exit_config_mode(&mut self, exit_command: Option<&str>) -> Result<(), NetsshError>;
    fn save_config(&mut self) -> Result<String, NetsshError>;
    fn send_command(&mut self, command: &str) -> Result<String, NetsshError>;
    fn change_context(&mut self, _context_name: &str) -> Result<(), NetsshError> {
        Ok(())
    }
}

// Removing redundant trait

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
