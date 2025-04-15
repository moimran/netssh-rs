use crate::device_connection::{DeviceInfo, NetworkDeviceConnection};
use crate::error::NetsshError;
use crate::vendors::cisco::{CiscoDeviceConnection, CiscoIosDevice};
use crate::vendors::common::DefaultConfigSetMethods;
use async_trait::async_trait;

#[async_trait]
impl NetworkDeviceConnection for CiscoIosDevice {
    fn connect(&mut self) -> Result<(), NetsshError> {
        self.connect()
    }

    fn close(&mut self) -> Result<(), NetsshError> {
        self.close()
    }

    fn check_config_mode(&mut self) -> Result<bool, NetsshError> {
        <Self as CiscoDeviceConnection>::check_config_mode(self)
    }

    fn enter_config_mode(&mut self, config_command: Option<&str>) -> Result<(), NetsshError> {
        <Self as CiscoDeviceConnection>::config_mode(self, config_command)
    }

    fn exit_config_mode(&mut self, exit_command: Option<&str>) -> Result<(), NetsshError> {
        <Self as CiscoDeviceConnection>::exit_config_mode(self, exit_command)
    }

    fn session_preparation(&mut self) -> Result<(), NetsshError> {
        <Self as CiscoDeviceConnection>::session_preparation(self)
    }

    fn terminal_settings(&mut self) -> Result<(), NetsshError> {
        <Self as CiscoDeviceConnection>::terminal_settings(self)
    }

    fn set_terminal_width(&mut self, width: u32) -> Result<(), NetsshError> {
        <Self as CiscoDeviceConnection>::set_terminal_width(self, width)
    }

    fn disable_paging(&mut self) -> Result<(), NetsshError> {
        <Self as CiscoDeviceConnection>::disable_paging(self)
    }

    fn set_base_prompt(&mut self) -> Result<String, NetsshError> {
        <Self as CiscoDeviceConnection>::set_base_prompt(self)
    }

    fn save_configuration(&mut self) -> Result<(), NetsshError> {
        <Self as CiscoDeviceConnection>::save_config(self).map(|_| ())
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
        <Self as CiscoDeviceConnection>::send_command(
            self,
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

    fn get_device_type(&self) -> &str {
        "cisco_ios"
    }

    fn send_config_commands(&mut self, commands: &[&str]) -> Result<Vec<String>, NetsshError> {
        let mut results = Vec::new();

        <Self as NetworkDeviceConnection>::enter_config_mode(self, None)?;

        for cmd in commands {
            let result = <Self as NetworkDeviceConnection>::send_command(
                self, cmd, None, // expect_string
                None, // read_timeout
                None, // auto_find_prompt
                None, // strip_prompt
                None, // strip_command
                None, // normalize
                None, // cmd_verify
            )?;
            results.push(result);
        }

        <Self as NetworkDeviceConnection>::exit_config_mode(self, None)?;

        Ok(results)
    }

    fn get_device_info(&mut self) -> Result<DeviceInfo, NetsshError> {
        let output = <Self as NetworkDeviceConnection>::send_command(
            self,
            "show version",
            None, // expect_string
            None, // read_timeout
            None, // auto_find_prompt
            None, // strip_prompt
            None, // strip_command
            None, // normalize
            None, // cmd_verify
        )?;

        let mut info = DeviceInfo {
            device_type: "cisco_ios".to_string(),
            hostname: "unknown".to_string(),
            version: "unknown".to_string(),
            model: "unknown".to_string(),
            serial: "unknown".to_string(),
            uptime: "unknown".to_string(),
        };

        for line in output.lines() {
            if line.contains("IOS Software") {
                info.version = line.trim().to_string();
            } else if line.contains("uptime is") {
                info.uptime = line.trim().to_string();
            } else if line.contains("processor") && line.contains("with") {
                info.model = line.trim().to_string();
            }
        }

        Ok(info)
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
        self.default_send_config_set(
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
