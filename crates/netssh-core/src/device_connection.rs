use crate::error::NetsshError;
use async_trait::async_trait;
use std::fmt::{Debug, Display};
use std::str::FromStr;
use std::time::Duration;

/// Information about a network device
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    /// Type of the device (vendor/OS)
    pub device_type: String,
    /// Hostname of the device
    pub hostname: String,
    /// Software version running on the device
    pub version: String,
    /// Model of the device
    pub model: String,
    /// Serial number of the device
    pub serial: String,
    /// Uptime of the device
    pub uptime: String,
}

/// Configuration for device connections
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct DeviceConfig {
    /// Hostname or IP address of the device
    pub host: String,
    /// Username for authentication
    pub username: String,
    /// Password for authentication
    pub password: Option<String>,
    /// Type of device (e.g., cisco_ios, juniper_junos)
    pub device_type: String,
    /// SSH port (default: 22)
    pub port: Option<u16>,
    /// Connection timeout in seconds
    pub timeout: Option<Duration>,
    /// Enable secret or privileged mode password
    pub secret: Option<String>,
    /// Whether to enable session logging
    pub session_log: Option<String>,
}

/// Different types of network devices supported by this library
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceType {
    /// Cisco IOS device
    CiscoIos,
    /// Cisco IOS XR device
    CiscoXr,
    /// Cisco NX-OS device
    CiscoNxos,
    /// Cisco ASA device
    CiscoAsa,
    /// Juniper Junos device
    JuniperJunos,
    /// Unknown device type
    Unknown,
}

impl FromStr for DeviceType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "cisco_ios" => Ok(DeviceType::CiscoIos),
            "cisco_xr" => Ok(DeviceType::CiscoXr),
            "cisco_nxos" => Ok(DeviceType::CiscoNxos),
            "cisco_asa" => Ok(DeviceType::CiscoAsa),
            "juniper_junos" => Ok(DeviceType::JuniperJunos),
            _ => Err(format!("Unknown device type: {}", s)),
        }
    }
}

impl Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceType::CiscoIos => write!(f, "cisco_ios"),
            DeviceType::CiscoXr => write!(f, "cisco_xr"),
            DeviceType::CiscoNxos => write!(f, "cisco_nxos"),
            DeviceType::CiscoAsa => write!(f, "cisco_asa"),
            DeviceType::JuniperJunos => write!(f, "juniper_junos"),
            DeviceType::Unknown => write!(f, "unknown"),
        }
    }
}

/// Trait defining the interface for network device connections
#[async_trait]
pub trait NetworkDeviceConnection: Send {
    /// Connect to the device
    fn connect(&mut self) -> Result<(), NetsshError>;

    /// Close the connection to the device
    fn close(&mut self) -> Result<(), NetsshError>;

    /// Send a command to the device and return the output
    ///
    /// This method sends a command to the device and returns the output.
    /// It includes optional parameters to customize command behavior.
    ///
    /// # Arguments
    /// * `command` - The command string to send to the device
    /// * `expect_string` - Optional pattern to search for in the output
    /// * `read_timeout` - Optional timeout in seconds for reading output
    /// * `auto_find_prompt` - Optional flag to automatically find prompt
    /// * `strip_prompt` - Optional flag to strip prompt from output
    /// * `strip_command` - Optional flag to strip command from output
    /// * `normalize` - Optional flag to normalize line feeds
    /// * `cmd_verify` - Optional flag to verify command echoing
    fn send_command(
        &mut self,
        command: &str,
        expect_string: Option<&str>,
        read_timeout: Option<f64>,
        auto_find_prompt: Option<bool>,
        strip_prompt: Option<bool>,
        strip_command: Option<bool>,
        normalize: Option<bool>,
        cmd_verify: Option<bool>,
    ) -> Result<String, NetsshError>;

    /// Send multiple configuration commands to the device
    fn send_config_commands(&mut self, commands: &[&str]) -> Result<Vec<String>, NetsshError>;

    /// Send a set of configuration commands with extended options
    ///
    /// This method provides a flexible way to send configuration commands
    /// with various options for verification, error handling, and output processing.
    fn send_config_set(
        &mut self,
        config_commands: Vec<String>,
        exit_config_mode: Option<bool>,
        read_timeout: Option<f64>,
        strip_prompt: Option<bool>,
        strip_command: Option<bool>,
        config_mode_command: Option<&str>,
        cmd_verify: Option<bool>,
        enter_config_mode: Option<bool>,
        error_pattern: Option<&str>,
        terminator: Option<&str>,
        bypass_commands: Option<&str>,
        fast_cli: Option<bool>,
    ) -> Result<String, NetsshError>;

    /// Get device-specific information
    fn get_device_info(&mut self) -> Result<DeviceInfo, NetsshError>;

    /// Get the type of the device
    fn get_device_type(&self) -> &str;

    /// Check if the device is in configuration mode
    fn check_config_mode(&mut self) -> Result<bool, NetsshError>;

    /// Enter configuration mode on the device
    fn enter_config_mode(&mut self, config_command: Option<&str>) -> Result<(), NetsshError>;

    /// Exit configuration mode on the device
    fn exit_config_mode(&mut self, exit_command: Option<&str>) -> Result<(), NetsshError>;

    /// Prepare the session with device-specific settings
    fn session_preparation(&mut self) -> Result<(), NetsshError>;

    /// Configure terminal settings
    fn terminal_settings(&mut self) -> Result<(), NetsshError>;

    /// Set the terminal width
    fn set_terminal_width(&mut self, width: u32) -> Result<(), NetsshError>;

    /// Disable paging on the device
    fn disable_paging(&mut self) -> Result<(), NetsshError>;

    /// Set the base prompt pattern
    fn set_base_prompt(&mut self) -> Result<String, NetsshError>;

    /// Save the device configuration
    fn save_configuration(&mut self) -> Result<(), NetsshError>;
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
