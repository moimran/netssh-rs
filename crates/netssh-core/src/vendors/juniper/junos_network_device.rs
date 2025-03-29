use crate::device_connection::{DeviceInfo, NetworkDeviceConnection};
use crate::error::NetsshError;
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
        <Self as JuniperDeviceConnection>::commit_config(self)
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
}
