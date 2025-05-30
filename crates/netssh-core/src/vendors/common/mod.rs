use crate::base_connection::BaseConnection;
use crate::error::NetsshError;

/// Provides default implementations for common configuration methods
pub trait DefaultConfigSetMethods {
    /// Gets the underlying BaseConnection
    fn get_base_connection(&mut self) -> &mut BaseConnection;

    /// Default implementation of send_config_set that delegates to the BaseConnection
    fn default_send_config_set(
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
        // Delegate to the BaseConnection's implementation
        self.get_base_connection().send_config_set_internal(
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