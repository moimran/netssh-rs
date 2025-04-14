use crate::base_connection::BaseConnection;
use crate::error::NetsshError;
use crate::vendors::common::DefaultConfigSetMethods;
use crate::vendors::juniper::{
    JuniperBaseConnection, JuniperDeviceConfig, JuniperDeviceConnection,
};
use async_trait::async_trait;
use tracing::debug;

pub struct JuniperJunosDevice {
    pub base: JuniperBaseConnection,
}

impl JuniperJunosDevice {
    pub fn new(config: JuniperDeviceConfig) -> Result<Self, NetsshError> {
        Ok(Self {
            base: JuniperBaseConnection::new(config)?,
        })
    }

    pub fn with_connection(connection: BaseConnection, config: JuniperDeviceConfig) -> Self {
        Self {
            base: JuniperBaseConnection::with_connection(connection, config),
        }
    }

    pub fn connect(&mut self) -> Result<(), NetsshError> {
        debug!(target: "JuniperJunosDevice::connect", "Connecting to Juniper JunOS device");
        self.base.connect()
    }

    pub fn close(&mut self) -> Result<(), NetsshError> {
        debug!(target: "JuniperJunosDevice::close", "Delegating to JuniperBaseConnection::close");
        self.base.close()
    }

    pub fn strip_ansi_escape_codes(&self, data: &str) -> String {
        self.base.strip_ansi_escape_codes(data)
    }

    // Re-export trait methods as inherent methods
    pub fn session_preparation(&mut self) -> Result<(), NetsshError> {
        debug!(target: "JuniperJunosDevice::session_preparation", "Delegating to JuniperBaseConnection::session_preparation");
        self.base.session_preparation()
    }

    pub fn set_terminal_width(&mut self, width: u32) -> Result<(), NetsshError> {
        debug!(target: "JuniperJunosDevice::set_terminal_width", "Delegating to JuniperBaseConnection::set_terminal_width");
        self.base.set_terminal_width(width)
    }

    pub fn disable_paging(&mut self) -> Result<(), NetsshError> {
        debug!(target: "JuniperJunosDevice::disable_paging", "Delegating to JuniperBaseConnection::disable_paging");
        self.base.disable_paging()
    }

    pub fn set_base_prompt(&mut self) -> Result<String, NetsshError> {
        debug!(target: "JuniperJunosDevice::set_base_prompt", "Delegating to JuniperBaseConnection::set_base_prompt");
        self.base.set_base_prompt()
    }

    pub fn check_config_mode(&mut self) -> Result<bool, NetsshError> {
        debug!(target: "JuniperJunosDevice::check_config_mode", "Delegating to JuniperBaseConnection::check_config_mode");
        self.base.check_config_mode()
    }

    pub fn config_mode(&mut self, config_command: Option<&str>) -> Result<(), NetsshError> {
        debug!(target: "JuniperJunosDevice::config_mode", "Delegating to JuniperBaseConnection::config_mode");
        self.base.config_mode(config_command)
    }

    pub fn exit_config_mode(&mut self, exit_command: Option<&str>) -> Result<(), NetsshError> {
        debug!(target: "JuniperJunosDevice::exit_config_mode", "Delegating to JuniperBaseConnection::exit_config_mode");
        self.base.exit_config_mode(exit_command)
    }

    pub fn commit_config(&mut self) -> Result<String, NetsshError> {
        debug!(target: "JuniperJunosDevice::commit_config", "Delegating to JuniperBaseConnection::commit_config");
        self.base.commit_config()
    }

    pub fn send_command(&mut self, command: &str) -> Result<String, NetsshError> {
        debug!(target: "JuniperJunosDevice::send_command", "Delegating to JuniperBaseConnection::send_command");
        self.base.send_command(command)
    }

    pub fn terminal_settings(&mut self) -> Result<(), NetsshError> {
        debug!(target: "JuniperJunosDevice::terminal_settings", "Delegating to JuniperBaseConnection::terminal_settings");
        self.base.terminal_settings()
    }

    // JunOS-specific methods
    pub fn show_version(&mut self) -> Result<String, NetsshError> {
        debug!(target: "JuniperJunosDevice::show_version", "Getting JunOS version");
        self.send_command("show version")
    }

    pub fn show_interfaces(&mut self) -> Result<String, NetsshError> {
        debug!(target: "JuniperJunosDevice::show_interfaces", "Getting JunOS interfaces");
        self.send_command("show interfaces terse")
    }
}

impl DefaultConfigSetMethods for JuniperJunosDevice {
    fn get_base_connection(&mut self) -> &mut BaseConnection {
        &mut self.base.connection
    }
}

#[async_trait]
impl JuniperDeviceConnection for JuniperJunosDevice {
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

    fn commit_config(&mut self) -> Result<String, NetsshError> {
        self.commit_config()
    }

    fn send_command(&mut self, command: &str) -> Result<String, NetsshError> {
        self.send_command(command)
    }
}
