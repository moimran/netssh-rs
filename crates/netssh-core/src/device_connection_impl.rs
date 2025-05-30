use crate::device_connection::{DeviceInfo, NetworkDeviceConnection};
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
    ) -> Result<String, NetsshError> {
        (**self).send_command_internal(
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
        (**self).get_device_type()
    }



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
    ) -> Result<String, NetsshError> {
        (**self).send_config_set_internal(
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

    fn get_device_info(&mut self) -> Result<DeviceInfo, NetsshError> {
        (**self).get_device_info()
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
    ) -> Result<String, NetsshError> {
        (**self).send_command_internal(
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
        (**self).get_device_type()
    }



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
    ) -> Result<String, NetsshError> {
        (**self).send_config_set_internal(
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

    fn get_device_info(&mut self) -> Result<DeviceInfo, NetsshError> {
        (**self).get_device_info()
    }
}
