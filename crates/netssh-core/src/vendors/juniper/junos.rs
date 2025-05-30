use crate::base_connection::BaseConnection;
use crate::device_connection::DeviceType;
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
        let mut base = JuniperBaseConnection::new(config)?;

        // Explicitly set the device type
        base.connection.set_device_type(DeviceType::JuniperJunos);

        Ok(Self { base })
    }

    pub fn with_connection(mut connection: BaseConnection, config: JuniperDeviceConfig) -> Self {
        // Explicitly set the device type
        connection.set_device_type(DeviceType::JuniperJunos);

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
        return self.base.commit_config();
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
        self.base.connection.send_command_internal(
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
        debug!(target: "JuniperJunosDevice::terminal_settings", "Delegating to JuniperBaseConnection::terminal_settings");
        self.base.terminal_settings()
    }

    // JunOS-specific methods
    pub fn show_version(&mut self) -> Result<String, NetsshError> {
        debug!(target: "JuniperJunosDevice::show_version", "Getting JunOS version");
        self.send_command(
            "show version",
            None, // expect_string
            None, // read_timeout
            None, // auto_find_prompt
            None, // strip_prompt
            None, // strip_command
            None, // normalize
            None, // cmd_verify
        )
    }

    pub fn show_interfaces(&mut self) -> Result<String, NetsshError> {
        debug!(target: "JuniperJunosDevice::show_interfaces", "Getting JunOS interfaces");
        self.send_command(
            "show interfaces terse",
            None, // expect_string
            None, // read_timeout
            None, // auto_find_prompt
            None, // strip_prompt
            None, // strip_command
            None, // normalize
            None, // cmd_verify
        )
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
        return self.commit_config();
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
        // Call the base connection's send_command method directly
        self.base.connection.send_command_internal(
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
}
