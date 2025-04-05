use crate::error::NetsshError;
use crate::vendors::common::DefaultConfigSetMethods;
use async_trait::async_trait;

pub mod juniperdevicebase;
pub mod junos;
pub mod junos_network_device;

pub use juniperdevicebase::JuniperBaseConnection;
pub use junos::JuniperJunosDevice;

#[derive(Clone, Debug)]
pub struct JuniperDeviceConfig {
    pub host: String,
    pub username: String,
    pub password: Option<String>,
    pub port: Option<u16>,
    pub timeout: Option<std::time::Duration>,
    pub secret: Option<String>,
    pub session_log: Option<String>,
}

impl Default for JuniperDeviceConfig {
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

/// Defines the interface for Juniper device interactions
pub trait JuniperDeviceConnection: DefaultConfigSetMethods {
    /// Prepare the session after connection
    fn session_preparation(&mut self) -> Result<(), NetsshError>;

    /// Configure terminal settings
    fn terminal_settings(&mut self) -> Result<(), NetsshError>;

    /// Set terminal width
    fn set_terminal_width(&mut self, width: u32) -> Result<(), NetsshError>;

    /// Disable paging
    fn disable_paging(&mut self) -> Result<(), NetsshError>;

    /// Set base prompt
    fn set_base_prompt(&mut self) -> Result<String, NetsshError>;

    /// Check if device is in configuration mode
    fn check_config_mode(&mut self) -> Result<bool, NetsshError>;

    /// Enter configuration mode
    fn config_mode(&mut self, config_command: Option<&str>) -> Result<(), NetsshError>;

    /// Exit configuration mode
    fn exit_config_mode(&mut self, exit_command: Option<&str>) -> Result<(), NetsshError>;

    /// Save configuration
    fn commit_config(&mut self) -> Result<(), NetsshError>;

    /// Send command to device
    fn send_command(&mut self, command: &str) -> Result<String, NetsshError>;
}
