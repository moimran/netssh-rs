use crate::device_connection::{DeviceInfo, NetworkDeviceConnection};
use crate::error::NetsshError;
use crate::vendors::juniper::{JuniperDeviceConnection, JuniperJunosDevice};
use async_trait::async_trait;
use tracing::debug;

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
        <Self as JuniperDeviceConnection>::send_command(
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
        "juniper_junos"
    }

    fn send_config_commands(&mut self, commands: &[&str]) -> Result<Vec<String>, NetsshError> {
        let mut results = Vec::new();

        <Self as NetworkDeviceConnection>::enter_config_mode(self, None)?;

        for cmd in commands {
            let result = <Self as NetworkDeviceConnection>::send_command(
                self, 
                cmd,
                None, // expect_string
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
        // Get the configuration output
        debug!(target: "JuniperJunosDevice::send_config_set", "Sending configuration set");

        // Initialize an output buffer to collect all outputs
        let mut output_buffer = String::new();

        // Store the result of sending config commands
        let config_result = self.base.connection.send_config_set(
            config_commands,
            Some(false), // Don't exit config mode after send_config_set
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
        );

        // Track if we had an error and capture output
        let mut had_error;

        // Process based on result
        match config_result {
            Ok(result) => {
                // Add command output to buffer
                output_buffer.push_str(&result);
                had_error = false;

                // If commands succeeded, commit the configuration
                debug!(target: "JuniperJunosDevice::send_config_set", "Commands succeeded, committing configuration");
                match <Self as JuniperDeviceConnection>::commit_config(self) {
                    Ok(commit_output) => {
                        debug!(target: "JuniperJunosDevice::send_config_set", "Commit output: {}", commit_output);
                        // Add commit output to buffer
                        // output_buffer.push_str("\n");
                        output_buffer.push_str(&commit_output);
                    }
                    Err(_commit_err) => {
                        // Save original error to return later
                        had_error = true;

                        // Try to rollback since commit failed
                        match self.send_command(
                            "rollback 0",
                            None, // expect_string
                            None, // read_timeout
                            None, // auto_find_prompt
                            None, // strip_prompt
                            None, // strip_command
                            None, // normalize
                            None, // cmd_verify
                        ) {
                            Ok(rollback_output) => {
                                debug!(target: "JuniperJunosDevice::send_config_set", "Rollback after failed commit: {}", rollback_output);
                                // output_buffer.push_str("\n");
                                output_buffer.push_str(&rollback_output);
                            }
                            Err(rollback_err) => {
                                debug!(target: "JuniperJunosDevice::send_config_set", "Rollback after failed commit failed: {}", rollback_err);
                                // Don't add rollback error to output
                            }
                        }
                    }
                }
            }
            Err(err) => {
                had_error = true;

                // Extract output from the error if it's a CommandErrorWithOutput variant
                match &err {
                    NetsshError::CommandErrorWithOutput { output, .. } => {
                        output_buffer = output.clone();
                    }
                    _ => {
                        // For other error types, just use the error message
                        debug!(target: "JuniperJunosDevice::send_config_set", "Error without output: {}", err);
                    }
                }

                // If commands failed, rollback configuration
                debug!(target: "JuniperJunosDevice::send_config_set", "Error occurred, performing rollback: {}", err);
                match self.send_command(
                    "rollback 0",
                    None, // expect_string
                    None, // read_timeout
                    None, // auto_find_prompt
                    None, // strip_prompt
                    None, // strip_command
                    None, // normalize
                    None, // cmd_verify
                ) {
                    Ok(rollback_output) => {
                        debug!(target: "JuniperJunosDevice::send_config_set", "Rollback successful: {}", rollback_output);
                        output_buffer.push_str("\n");
                        output_buffer.push_str(&rollback_output);
                    }
                    Err(rollback_err) => {
                        debug!(target: "JuniperJunosDevice::send_config_set", "Rollback failed: {}", rollback_err);
                        // Don't add rollback error to output
                    }
                }
            }
        };

        // Always exit config mode, regardless of success or failure
        debug!(target: "JuniperJunosDevice::send_config_set", "Exiting config mode");
        if let Ok(()) = self.exit_config_mode(None) {
            debug!(target: "JuniperJunosDevice::send_config_set", "Successfully exited config mode");
        }

        // Return based on whether we had an error
        if had_error {
            Err(NetsshError::command_error_with_output(
                "Configuration command(s) failed".to_string(),
                output_buffer,
            ))
        } else {
            Ok(output_buffer)
        }
    }
}
