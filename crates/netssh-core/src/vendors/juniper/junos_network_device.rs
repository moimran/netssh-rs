use crate::device_connection::{NetworkDeviceConnection};
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
}
