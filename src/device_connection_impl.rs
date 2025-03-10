use crate::device_connection::NetworkDeviceConnection;
use crate::error::NetsshError;
use async_trait::async_trait;

#[async_trait]
impl NetworkDeviceConnection for Box<dyn NetworkDeviceConnection> {
    fn connect(&mut self) -> Result<(), NetsshError> {
        (**self).connect()
    }
    
    fn close(&mut self) -> Result<(), NetsshError> {
        (**self).close()
    }
    
    fn check_config_mode(&mut self) -> Result<bool, NetsshError> {
        (**self).check_config_mode()
    }
    
    fn enter_config_mode(&mut self, config_command: Option<&str>) -> Result<(), NetsshError> {
        (**self).enter_config_mode(config_command)
    }
    
    fn exit_config_mode(&mut self, exit_command: Option<&str>) -> Result<(), NetsshError> {
        (**self).exit_config_mode(exit_command)
    }
    
    fn session_preparation(&mut self) -> Result<(), NetsshError> {
        (**self).session_preparation()
    }
    
    fn terminal_settings(&mut self) -> Result<(), NetsshError> {
        (**self).terminal_settings()
    }
    
    fn set_terminal_width(&mut self, width: u32) -> Result<(), NetsshError> {
        (**self).set_terminal_width(width)
    }
    
    fn disable_paging(&mut self) -> Result<(), NetsshError> {
        (**self).disable_paging()
    }
    
    fn set_base_prompt(&mut self) -> Result<String, NetsshError> {
        (**self).set_base_prompt()
    }
    
    fn save_configuration(&mut self) -> Result<(), NetsshError> {
        (**self).save_configuration()
    }
    
    fn send_command(&mut self, command: &str) -> Result<String, NetsshError> {
        (**self).send_command(command)
    }
    
    fn get_device_type(&self) -> &str {
        (**self).get_device_type()
    }
}

#[async_trait]
impl NetworkDeviceConnection for Box<dyn NetworkDeviceConnection + Send> {
    fn connect(&mut self) -> Result<(), NetsshError> {
        (**self).connect()
    }
    
    fn close(&mut self) -> Result<(), NetsshError> {
        (**self).close()
    }
    
    fn check_config_mode(&mut self) -> Result<bool, NetsshError> {
        (**self).check_config_mode()
    }
    
    fn enter_config_mode(&mut self, config_command: Option<&str>) -> Result<(), NetsshError> {
        (**self).enter_config_mode(config_command)
    }
    
    fn exit_config_mode(&mut self, exit_command: Option<&str>) -> Result<(), NetsshError> {
        (**self).exit_config_mode(exit_command)
    }
    
    fn session_preparation(&mut self) -> Result<(), NetsshError> {
        (**self).session_preparation()
    }
    
    fn terminal_settings(&mut self) -> Result<(), NetsshError> {
        (**self).terminal_settings()
    }
    
    fn set_terminal_width(&mut self, width: u32) -> Result<(), NetsshError> {
        (**self).set_terminal_width(width)
    }
    
    fn disable_paging(&mut self) -> Result<(), NetsshError> {
        (**self).disable_paging()
    }
    
    fn set_base_prompt(&mut self) -> Result<String, NetsshError> {
        (**self).set_base_prompt()
    }
    
    fn save_configuration(&mut self) -> Result<(), NetsshError> {
        (**self).save_configuration()
    }
    
    fn send_command(&mut self, command: &str) -> Result<String, NetsshError> {
        (**self).send_command(command)
    }
    
    fn get_device_type(&self) -> &str {
        (**self).get_device_type()
    }
}
