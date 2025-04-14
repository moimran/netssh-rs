use crate::device_connection::{DeviceInfo, NetworkDeviceConnection};
use crate::error::NetsshError;
use crate::vendors::common::DefaultConfigSetMethods;
use crate::vendors::juniper::{JuniperDeviceConnection, JuniperJunosDevice};
use async_trait::async_trait;

#[async_trait]
impl NetworkDeviceConnection for JuniperJunosDevice {
    fn connect(&mut self) -> Result<(), NetsshError> {
        self.connect()
    }

    fn close(&mut self) -> Result<(), NetsshError> {
        self.close()
    }

    fn check_config_mode(&mut self) -> Result<bool, NetsshError> {
        <Self as JuniperDeviceConnection>::check_config_mode(self)
    }

    fn enter_config_mode(&mut self, config_command: Option<&str>) -> Result<(), NetsshError> {
        <Self as JuniperDeviceConnection>::config_mode(self, config_command)
    }

    fn exit_config_mode(&mut self, exit_command: Option<&str>) -> Result<(), NetsshError> {
        <Self as JuniperDeviceConnection>::exit_config_mode(self, exit_command)
    }

    fn session_preparation(&mut self) -> Result<(), NetsshError> {
        <Self as JuniperDeviceConnection>::session_preparation(self)
    }

    fn terminal_settings(&mut self) -> Result<(), NetsshError> {
        <Self as JuniperDeviceConnection>::terminal_settings(self)
    }

    fn set_terminal_width(&mut self, width: u32) -> Result<(), NetsshError> {
        <Self as JuniperDeviceConnection>::set_terminal_width(self, width)
    }

    fn disable_paging(&mut self) -> Result<(), NetsshError> {
        <Self as JuniperDeviceConnection>::disable_paging(self)
    }

    fn set_base_prompt(&mut self) -> Result<String, NetsshError> {
        <Self as JuniperDeviceConnection>::set_base_prompt(self)
    }

    fn save_configuration(&mut self) -> Result<(), NetsshError> {
        let _ = <Self as JuniperDeviceConnection>::commit_config(self)?;
        Ok(())
    }

    fn send_command(&mut self, command: &str) -> Result<String, NetsshError> {
        <Self as JuniperDeviceConnection>::send_command(self, command)
    }

    fn get_device_type(&self) -> &str {
        "juniper_junos"
    }

    fn send_config_commands(&mut self, commands: &[&str]) -> Result<Vec<String>, NetsshError> {
        let mut results = Vec::new();

        <Self as NetworkDeviceConnection>::enter_config_mode(self, None)?;

        for cmd in commands {
            let result = <Self as NetworkDeviceConnection>::send_command(self, cmd)?;
            results.push(result);
        }

        <Self as NetworkDeviceConnection>::exit_config_mode(self, None)?;

        Ok(results)
    }

    fn get_device_info(&mut self) -> Result<DeviceInfo, NetsshError> {
        let output = <Self as NetworkDeviceConnection>::send_command(self, "show version")?;

        let mut info = DeviceInfo {
            device_type: "juniper_junos".to_string(),
            hostname: "unknown".to_string(),
            version: "unknown".to_string(),
            model: "unknown".to_string(),
            serial: "unknown".to_string(),
            uptime: "unknown".to_string(),
        };

        for line in output.lines() {
            if line.contains("Junos:") {
                info.version = line.trim().to_string();
            } else if line.contains("uptime:") {
                info.uptime = line.trim().to_string();
            } else if line.contains("Model:") {
                info.model = line.trim().to_string();
            }
        }

        Ok(info)
    }

    fn send_config_set(
        &mut self,
        config_commands: Vec<String>,
        _exit_config_mode: Option<bool>,
        read_timeout: Option<f64>,
        strip_prompt: Option<bool>,
        strip_command: Option<bool>,
        _config_mode_command: Option<&str>,
        cmd_verify: Option<bool>,
        _enter_config_mode: Option<bool>,
        error_pattern: Option<&str>,
        terminator: Option<&str>,
        bypass_commands: Option<&str>,
        fast_cli: Option<bool>,
    ) -> Result<String, NetsshError> {
        let result = self.default_send_config_set(
            config_commands,
            Some(false),
            read_timeout,
            strip_prompt,
            strip_command,
            Some("configure"),
            cmd_verify,
            Some(true),
            error_pattern,
            terminator,
            bypass_commands,
            fast_cli,
        )?;

        let commit_output = <Self as JuniperDeviceConnection>::commit_config(self)?;
        self.exit_config_mode(None)?;

        Ok(format!("{}\n{}", result, commit_output))
    }
}
