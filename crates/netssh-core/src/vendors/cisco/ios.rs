use crate::base_connection::BaseConnection;
use crate::error::NetsshError;
use crate::vendors::cisco::{CiscoBaseConnection, CiscoDeviceConfig, CiscoDeviceConnection};
use crate::vendors::common::DefaultConfigSetMethods;
use async_trait::async_trait;
use tracing::debug;

pub struct CiscoIosDevice {
    pub base: CiscoBaseConnection,
}

impl CiscoIosDevice {
    pub fn new(config: CiscoDeviceConfig) -> Result<Self, NetsshError> {
        Ok(Self {
            base: CiscoBaseConnection::new(config)?,
        })
    }

    pub fn with_connection(connection: BaseConnection, config: CiscoDeviceConfig) -> Self {
        Self {
            base: CiscoBaseConnection::with_connection(connection, config),
        }
    }

    pub fn connect(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoIosDevice::connect", "Delegating to CiscoBaseConnection::connect");
        self.base.connect()
    }

    pub fn check_enable_mode(&mut self) -> Result<bool, NetsshError> {
        debug!(target: "CiscoIosDevice::check_enable_mode", "Delegating to CiscoBaseConnection::check_enable_mode");
        self.base.check_enable_mode()
    }

    pub fn enable(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoIosDevice::enable", "Delegating to CiscoBaseConnection::enable");
        self.base.enable()
    }

    pub fn exit_enable_mode(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoIosDevice::exit_enable_mode", "Delegating to CiscoBaseConnection::exit_enable_mode");
        self.base.exit_enable_mode()
    }

    pub fn close(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoIosDevice::close", "Delegating to CiscoBaseConnection::close");
        self.base.close()
    }

    pub fn strip_ansi_escape_codes(&self, data: &str) -> String {
        self.base.strip_ansi_escape_codes(data)
    }

    // Re-export trait methods as inherent methods
    pub fn session_preparation(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoIosDevice::session_preparation", "Delegating to CiscoBaseConnection::session_preparation");
        self.base.session_preparation()
    }

    pub fn set_terminal_width(&mut self, width: u32) -> Result<(), NetsshError> {
        debug!(target: "CiscoIosDevice::set_terminal_width", "Delegating to CiscoBaseConnection::set_terminal_width");
        self.base.set_terminal_width(width)
    }

    pub fn disable_paging(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoIosDevice::disable_paging", "Delegating to CiscoBaseConnection::disable_paging");
        self.base.disable_paging()
    }

    pub fn set_base_prompt(&mut self) -> Result<String, NetsshError> {
        debug!(target: "CiscoIosDevice::set_base_prompt", "Delegating to CiscoBaseConnection::set_base_prompt");
        self.base.set_base_prompt()
    }

    pub fn check_config_mode(&mut self) -> Result<bool, NetsshError> {
        debug!(target: "CiscoIosDevice::check_config_mode", "Delegating to CiscoBaseConnection::check_config_mode");
        self.base.check_config_mode()
    }

    pub fn config_mode(&mut self, config_command: Option<&str>) -> Result<(), NetsshError> {
        debug!(target: "CiscoIosDevice::config_mode", "Delegating to CiscoBaseConnection::config_mode");
        self.base.config_mode(config_command)
    }

    pub fn exit_config_mode(&mut self, exit_command: Option<&str>) -> Result<(), NetsshError> {
        debug!(target: "CiscoIosDevice::exit_config_mode", "Delegating to CiscoBaseConnection::exit_config_mode");
        self.base.exit_config_mode(exit_command)
    }

    pub fn save_config(&mut self) -> Result<String, NetsshError> {
        debug!(target: "CiscoIosDevice::save_config", "Delegating to CiscoBaseConnection::save_config");
        self.base.save_config()
    }

    pub fn send_command(
        &mut self,
        command: &str,
        expect_string: Option<&str>,
        read_timeout: Option<f64>,
        auto_find_prompt: Option<bool>,
        strip_prompt: Option<bool>,
        strip_command: Option<bool>,
        normalize: Option<bool>,
        cmd_verify: Option<bool>,
    ) -> Result<String, NetsshError> {
        debug!(target: "CiscoIosDevice::send_command", "Delegating to CiscoBaseConnection::send_command");
        self.base.send_command(
            command,
            expect_string,
            read_timeout,
            auto_find_prompt,
            strip_prompt,
            strip_command,
            normalize,
            cmd_verify,
        )
    }

    pub fn terminal_settings(&mut self) -> Result<(), NetsshError> {
        debug!(target: "CiscoIosDevice::terminal_settings", "Delegating to CiscoBaseConnection::terminal_settings");
        self.base.terminal_settings()
    }
}

impl DefaultConfigSetMethods for CiscoIosDevice {
    fn get_base_connection(&mut self) -> &mut BaseConnection {
        &mut self.base.connection
    }
}

#[async_trait]
impl CiscoDeviceConnection for CiscoIosDevice {
    fn session_preparation(&mut self) -> Result<(), NetsshError> {
        self.session_preparation()
    }

    fn set_terminal_width(&mut self, width: u32) -> Result<(), NetsshError> {
        self.set_terminal_width(width)
    }

    fn disable_paging(&mut self) -> Result<(), NetsshError> {
        self.disable_paging()
    }

    fn terminal_settings(&mut self) -> Result<(), NetsshError> {
        self.terminal_settings()
    }

    fn set_base_prompt(&mut self) -> Result<String, NetsshError> {
        self.set_base_prompt()
    }

    fn check_config_mode(&mut self) -> Result<bool, NetsshError> {
        self.check_config_mode()
    }

    fn config_mode(&mut self, config_command: Option<&str>) -> Result<(), NetsshError> {
        self.config_mode(config_command)
    }

    fn exit_config_mode(&mut self, exit_command: Option<&str>) -> Result<(), NetsshError> {
        self.exit_config_mode(exit_command)
    }

    fn save_config(&mut self) -> Result<String, NetsshError> {
        self.save_config()
    }

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
    ) -> Result<String, NetsshError> {
        self.send_command(
            command,
            expect_string,
            read_timeout,
            auto_find_prompt,
            strip_prompt,
            strip_command,
            normalize,
            cmd_verify,
        )
    }

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
    ) -> Result<String, NetsshError> {
        self.base.connection.send_config_set(
            config_commands,
            exit_config_mode,
            read_timeout,
            strip_prompt,
            strip_command,
            config_mode_command,
            cmd_verify,
            enter_config_mode,
            error_pattern,
            terminator,
            bypass_commands,
            fast_cli,
        )
    }
}
