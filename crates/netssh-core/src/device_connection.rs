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

/// Builder for send_command with fluent API
pub struct SendCommand<'a> {
    device: &'a mut dyn NetworkDeviceConnection,
    command: &'a str,
    expect_string: Option<&'a str>,
    read_timeout: Option<f64>,
    auto_find_prompt: Option<bool>,
    strip_prompt: Option<bool>,
    strip_command: Option<bool>,
    normalize: Option<bool>,
    cmd_verify: Option<bool>,
}

impl<'a> SendCommand<'a> {
    /// Create a new SendCommand builder
    pub fn new(device: &'a mut dyn NetworkDeviceConnection, command: &'a str) -> Self {
        Self {
            device,
            command,
            expect_string: None,
            read_timeout: None,
            auto_find_prompt: None,
            strip_prompt: None,
            strip_command: None,
            normalize: None,
            cmd_verify: None,
        }
    }

    /// Set the expected string to wait for (instead of prompt)
    pub fn expect_string(mut self, expect: &'a str) -> Self {
        self.expect_string = Some(expect);
        self
    }

    /// Set the read timeout in seconds (default: 10.0)
    pub fn timeout(mut self, timeout: f64) -> Self {
        self.read_timeout = Some(timeout);
        self
    }

    /// Set whether to automatically find the prompt (default: false)
    pub fn auto_find_prompt(mut self, auto_find: bool) -> Self {
        self.auto_find_prompt = Some(auto_find);
        self
    }

    /// Set whether to strip the prompt from output (default: true)
    pub fn strip_prompt(mut self, strip: bool) -> Self {
        self.strip_prompt = Some(strip);
        self
    }

    /// Set whether to strip the command echo from output (default: true)
    pub fn strip_command(mut self, strip: bool) -> Self {
        self.strip_command = Some(strip);
        self
    }

    /// Set whether to normalize line endings (default: true)
    pub fn normalize(mut self, normalize: bool) -> Self {
        self.normalize = Some(normalize);
        self
    }

    /// Set whether to verify command echoing (default: false)
    pub fn cmd_verify(mut self, verify: bool) -> Self {
        self.cmd_verify = Some(verify);
        self
    }

    /// Execute the command with the configured options
    pub fn execute(self) -> Result<String, NetsshError> {
        self.device.send_command_internal(
            self.command,
            self.expect_string,
            self.read_timeout,
            self.auto_find_prompt,
            self.strip_prompt,
            self.strip_command,
            self.normalize,
            self.cmd_verify,
        )
    }
}

/// Builder for send_config_set with fluent API
pub struct SendConfigSet<'a> {
    device: &'a mut dyn NetworkDeviceConnection,
    config_commands: Vec<String>,
    exit_config_mode: Option<bool>,
    read_timeout: Option<f64>,
    strip_prompt: Option<bool>,
    strip_command: Option<bool>,
    config_mode_command: Option<&'a str>,
    cmd_verify: Option<bool>,
    enter_config_mode: Option<bool>,
    error_pattern: Option<&'a str>,
    terminator: Option<&'a str>,
    bypass_commands: Option<&'a str>,
    fast_cli: Option<bool>,
}

impl<'a> SendConfigSet<'a> {
    /// Create a new SendConfigSet builder
    pub fn new(device: &'a mut dyn NetworkDeviceConnection, config_commands: Vec<String>) -> Self {
        Self {
            device,
            config_commands,
            exit_config_mode: None,
            read_timeout: None,
            strip_prompt: None,
            strip_command: None,
            config_mode_command: None,
            cmd_verify: None,
            enter_config_mode: None,
            error_pattern: None,
            terminator: None,
            bypass_commands: None,
            fast_cli: None,
        }
    }

    /// Set whether to exit config mode after completion (default: true)
    pub fn exit_config_mode(mut self, exit: bool) -> Self {
        self.exit_config_mode = Some(exit);
        self
    }

    /// Set the read timeout in seconds (default: 10.0)
    pub fn timeout(mut self, timeout: f64) -> Self {
        self.read_timeout = Some(timeout);
        self
    }

    /// Set whether to strip the prompt from output (default: true)
    pub fn strip_prompt(mut self, strip: bool) -> Self {
        self.strip_prompt = Some(strip);
        self
    }

    /// Set whether to strip the command echo from output (default: true)
    pub fn strip_command(mut self, strip: bool) -> Self {
        self.strip_command = Some(strip);
        self
    }

    /// Set the command to enter config mode
    pub fn config_mode_command(mut self, command: &'a str) -> Self {
        self.config_mode_command = Some(command);
        self
    }

    /// Set whether to verify command echoing (default: false)
    pub fn cmd_verify(mut self, verify: bool) -> Self {
        self.cmd_verify = Some(verify);
        self
    }

    /// Set whether to enter config mode before sending commands (default: true)
    pub fn enter_config_mode(mut self, enter: bool) -> Self {
        self.enter_config_mode = Some(enter);
        self
    }

    /// Set regex pattern to detect configuration errors
    pub fn error_pattern(mut self, pattern: &'a str) -> Self {
        self.error_pattern = Some(pattern);
        self
    }

    /// Set alternate terminator pattern
    pub fn terminator(mut self, term: &'a str) -> Self {
        self.terminator = Some(term);
        self
    }

    /// Set regex pattern for commands that should bypass verification
    pub fn bypass_commands(mut self, pattern: &'a str) -> Self {
        self.bypass_commands = Some(pattern);
        self
    }

    /// Set whether to use fast mode (minimal verification) (default: false)
    pub fn fast_cli(mut self, fast: bool) -> Self {
        self.fast_cli = Some(fast);
        self
    }

    /// Execute the configuration commands with the configured options
    pub fn execute(self) -> Result<String, NetsshError> {
        self.device.send_config_set_internal(
            self.config_commands,
            self.exit_config_mode,
            self.read_timeout,
            self.strip_prompt,
            self.strip_command,
            self.config_mode_command,
            self.cmd_verify,
            self.enter_config_mode,
            self.error_pattern,
            self.terminator,
            self.bypass_commands,
            self.fast_cli,
        )
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
    /// All parameters except the command have sensible defaults.
    ///
    /// # Arguments
    /// * `command` - The command string to send to the device
    /// * `expect_string` - Optional pattern to search for in the output (default: None)
    /// * `read_timeout` - Optional timeout in seconds for reading output (default: 10.0)
    /// * `auto_find_prompt` - Optional flag to automatically find prompt (default: false)
    /// * `strip_prompt` - Optional flag to strip prompt from output (default: true)
    /// * `strip_command` - Optional flag to strip command from output (default: true)
    /// * `normalize` - Optional flag to normalize line feeds (default: true)
    /// * `cmd_verify` - Optional flag to verify command echoing (default: false)
    ///
    /// # Examples
    /// ```rust
    /// // Simple command with all defaults
    /// let output = device.send_command("show version")?;
    ///
    /// // Command with custom timeout
    /// let output = device.send_command("show version", None, Some(30.0))?;
    ///
    /// // Command with custom timeout and no prompt stripping
    /// let output = device.send_command("show version", None, Some(30.0), None, Some(false))?;
    /// ```
    /// Send a command to the device using builder pattern
    ///
    /// This method returns a builder that allows you to configure command options
    /// using a fluent API. Only specify the options you need - all others use sensible defaults.
    ///
    /// # Examples
    /// ```rust
    /// // Simple command with all defaults
    /// let output = device.send_command("show version").execute()?;
    ///
    /// // Command with custom timeout
    /// let output = device.send_command("show version")
    ///     .timeout(30.0)
    ///     .execute()?;
    ///
    /// // Command with multiple options
    /// let output = device.send_command("show tech-support")
    ///     .timeout(120.0)
    ///     .strip_prompt(false)
    ///     .cmd_verify(true)
    ///     .execute()?;
    /// ```
    fn send_command<'a>(&'a mut self, command: &'a str) -> SendCommand<'a>
    where
        Self: Sized,
    {
        SendCommand::new(self, command)
    }

    /// Internal method for executing commands (used by builder)
    fn send_command_internal(
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



    /// Send configuration commands to the device using builder pattern
    ///
    /// This method returns a builder that allows you to configure options
    /// using a fluent API. Handles entering and exiting configuration mode automatically.
    ///
    /// # Examples
    /// ```rust
    /// let commands = vec!["interface eth0".to_string(), "ip address 192.168.1.1/24".to_string()];
    ///
    /// // Simple config with defaults
    /// let output = device.send_config_set(commands).execute()?;
    ///
    /// // Config with custom options
    /// let output = device.send_config_set(commands)
    ///     .timeout(60.0)
    ///     .exit_config_mode(false)
    ///     .execute()?;
    /// ```
    fn send_config_set<'a>(&'a mut self, config_commands: Vec<String>) -> SendConfigSet<'a>
    where
        Self: Sized,
    {
        SendConfigSet::new(self, config_commands)
    }

    /// Internal method for executing config commands (used by builder)
    fn send_config_set_internal(
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
