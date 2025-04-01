use crate::device_connection::{DeviceInfo, NetworkDeviceConnection};
use crate::error::NetsshError;
use crate::vendors::cisco::{CiscoDeviceConnection, CiscoNxosDevice};
use async_trait::async_trait;

#[async_trait]
impl NetworkDeviceConnection for CiscoNxosDevice {
    fn connect(&mut self) -> Result<(), NetsshError> {
        // Delegating to the device implementation
        CiscoNxosDevice::connect(self)
    }

    fn close(&mut self) -> Result<(), NetsshError> {
        // Delegating to the device implementation
        CiscoNxosDevice::close(self)
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

    fn send_command(&mut self, command: &str) -> Result<String, NetsshError> {
        <Self as CiscoDeviceConnection>::send_command(self, command)
    }

    fn session_preparation(&mut self) -> Result<(), NetsshError> {
        <Self as CiscoDeviceConnection>::session_preparation(self)
    }

    fn get_device_type(&self) -> &str {
        "cisco_nxos"
    }

    fn send_config_commands(&mut self, commands: &[&str]) -> Result<Vec<String>, NetsshError> {
        // Implementation for sending config commands to Cisco NXOS
        let mut results = Vec::new();

        // Enter config mode using explicit trait method
        <Self as NetworkDeviceConnection>::enter_config_mode(self, None)?;

        // Send each command using explicit trait method
        for cmd in commands {
            let result = <Self as NetworkDeviceConnection>::send_command(self, cmd)?;
            results.push(result);
        }

        // Exit config mode using explicit trait method
        <Self as NetworkDeviceConnection>::exit_config_mode(self, None)?;

        Ok(results)
    }

    fn get_device_info(&mut self) -> Result<DeviceInfo, NetsshError> {
        // Get device information using show version with explicit trait method
        let output = <Self as NetworkDeviceConnection>::send_command(self, "show version")?;

        // Parse the output to extract device information
        let mut info = DeviceInfo {
            device_type: "cisco_nxos".to_string(),
            hostname: "unknown".to_string(),
            version: "unknown".to_string(),
            model: "unknown".to_string(),
            serial: "unknown".to_string(),
            uptime: "unknown".to_string(),
        };

        // Extract information from output
        for line in output.lines() {
            if line.contains("NXOS:") {
                info.version = line.trim().to_string();
            } else if line.contains("uptime is") {
                info.uptime = line.trim().to_string();
            } else if line.contains("Hardware") {
                info.model = line.trim().to_string();
            }
        }

        Ok(info)
    }
}
