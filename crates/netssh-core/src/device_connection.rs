 use crate::error::NetsshError;
use async_trait::async_trait;

/// Common interface for all network devices regardless of vendor
#[async_trait]
pub trait NetworkDeviceConnection {
    /// Connect to the device
    fn connect(&mut self) -> Result<(), NetsshError>;
    
    /// Close the connection to the device
    fn close(&mut self) -> Result<(), NetsshError>;
    
    /// Check if the device is in configuration mode
    fn check_config_mode(&mut self) -> Result<bool, NetsshError>;
    
    /// Enter configuration mode
    fn enter_config_mode(&mut self, config_command: Option<&str>) -> Result<(), NetsshError>;
    
    /// Exit configuration mode
    fn exit_config_mode(&mut self, exit_command: Option<&str>) -> Result<(), NetsshError>;
    
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
    
    /// Save or commit configuration
    fn save_configuration(&mut self) -> Result<(), NetsshError>;
    
    /// Send command to device
    fn send_command(&mut self, command: &str) -> Result<String, NetsshError>;
    
    /// Get the device type (vendor and model)
    fn get_device_type(&self) -> &str;
}

/// Basic device information
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub vendor: String,
    pub model: String,
    pub os_version: String,
    pub hostname: String,
    pub uptime: String,
}

/// Generic device configuration
#[derive(Debug, Clone)]
pub struct DeviceConfig {
    pub device_type: String,
    pub host: String,
    pub username: String,
    pub password: Option<String>,
    pub port: Option<u16>,
    pub timeout: Option<std::time::Duration>,
    pub secret: Option<String>,
    pub session_log: Option<String>,
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            device_type: String::new(),
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
