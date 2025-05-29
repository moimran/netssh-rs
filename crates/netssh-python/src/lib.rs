use chrono::Utc;
use netssh_core::command_result::{BatchCommandResults, CommandResult};
use netssh_core::device_connection::{DeviceConfig, NetworkDeviceConnection};
use netssh_core::device_factory::DeviceFactory;
use netssh_core::error::NetsshError;
use netssh_core::{FailureStrategy, ParallelExecutionConfig, ParallelExecutionManager};
use pyo3::conversion::ToPyObject;
use pyo3::exceptions::{PyException, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pyo3::wrap_pyfunction;
use std::collections::HashMap;
use std::time::Duration;

// Define custom PyConnectionError
#[pyclass]
struct PyConnectionError {
    #[allow(dead_code)]
    msg: String,
}

#[pymethods]
impl PyConnectionError {
    #[new]
    fn new(msg: String) -> Self {
        Self { msg }
    }
}

// Function to convert NetsshError to PyErr and extract command output if available
fn netssh_error_to_pyerr(err: NetsshError) -> (PyErr, Option<String>) {
    let output = match &err {
        NetsshError::CommandErrorWithOutput { output, .. } => Some(output.clone()),
        _ => None,
    };

    let py_err = match err {
        NetsshError::ConnectionError(msg) => {
            PyException::new_err(format!("Connection error: {}", msg))
        }
        NetsshError::AuthenticationError(msg) => {
            PyException::new_err(format!("Authentication error: {}", msg))
        }
        NetsshError::CommandError(msg) => PyValueError::new_err(format!("Command error: {}", msg)),
        NetsshError::CommandErrorWithOutput { error_msg, .. } => {
            PyValueError::new_err(format!("Command error: {}", error_msg))
        }
        // Add other error variants as needed
        _ => PyRuntimeError::new_err(format!("Netssh error: {}", err)),
    };

    (py_err, output)
}

// Helper function that only returns the PyErr without the output
fn netssh_error_to_pyerr_simple(err: NetsshError) -> PyErr {
    netssh_error_to_pyerr(err).0
}

/// Python module for netssh-rs
#[pymodule]
fn netssh_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    // Register custom exception
    m.add_class::<PyConnectionError>()?;

    // Add classes
    m.add_class::<PyDeviceConfig>()?;
    m.add_class::<PyNetworkDevice>()?;
    m.add_class::<PyCommandResult>()?;
    m.add_class::<PyBatchCommandResults>()?;
    m.add_class::<PyParallelExecutionManager>()?;

    // Add functions
    m.add_function(wrap_pyfunction!(initialize_logging, m)?)?;
    m.add_function(wrap_pyfunction!(set_default_session_logging, m)?)?;

    Ok(())
}

/// Set default session logging behavior
///
/// This function configures whether session logging is enabled by default
/// and where the logs are stored when no specific path is provided.
#[pyfunction]
#[pyo3(signature = (enable=false, log_path=None))]
#[pyo3(text_signature = "(enable=False, log_path=None)")]
fn set_default_session_logging(enable: bool, log_path: Option<&str>) -> PyResult<()> {
    // Update the global settings
    netssh_core::settings::Settings::update(|settings| {
        settings.logging.enable_session_log = enable;

        if let Some(path) = log_path {
            settings.logging.session_log_path = path.to_string();
        }
    })
    .map_err(|e| PyRuntimeError::new_err(format!("Failed to update settings: {}", e)))
}

/// Initialize logging
#[pyfunction]
#[pyo3(signature = (level="info", log_to_file=false, log_file_path=None, log_format=None))]
#[pyo3(text_signature = "(level='info', log_to_file=False, log_file_path=None, log_format=None)")]
fn initialize_logging(
    level: &str,
    log_to_file: bool,
    log_file_path: Option<&str>,
    log_format: Option<&str>,
) -> PyResult<()> {
    netssh_core::init_logging(level, log_to_file, log_file_path, log_format)
        .map_err(netssh_error_to_pyerr_simple)
}

/// Python wrapper for DeviceConfig
///
/// This class represents the configuration for connecting to a network device.
/// It contains all the necessary information to establish an SSH connection.
#[pyclass]
#[derive(Clone)]
struct PyDeviceConfig {
    #[pyo3(get, set)]
    device_type: String,
    #[pyo3(get, set)]
    host: String,
    #[pyo3(get, set)]
    username: String,
    #[pyo3(get, set)]
    password: Option<String>,
    #[pyo3(get, set)]
    port: Option<u16>,
    #[pyo3(get, set)]
    timeout_seconds: Option<u64>,
    #[pyo3(get, set)]
    secret: Option<String>,
    #[pyo3(get, set)]
    session_log: Option<String>,
}

#[pymethods]
impl PyDeviceConfig {
    #[new]
    #[pyo3(signature = (device_type, host, username, password=None, port=None, timeout_seconds=None, secret=None, session_log=None))]
    #[pyo3(
        text_signature = "(device_type, host, username, password=None, port=None, timeout_seconds=None, secret=None, session_log=None)"
    )]
    /// Create a new device configuration.
    ///
    /// Args:
    ///     device_type: The type of device (e.g., 'cisco_ios', 'juniper')
    ///     host: The hostname or IP address of the device
    ///     username: The username for authentication
    ///     password: The password for authentication (optional)
    ///     port: The SSH port (default: 22)
    ///     timeout_seconds: Connection timeout in seconds (default: 60)
    ///     secret: The enable secret for privileged mode (if required)
    ///     session_log: Path to log file for session logging (optional)
    ///
    /// Returns:
    ///     A new PyDeviceConfig instance
    fn new(
        device_type: String,
        host: String,
        username: String,
        password: Option<String>,
        port: Option<u16>,
        timeout_seconds: Option<u64>,
        secret: Option<String>,
        session_log: Option<String>,
    ) -> Self {
        Self {
            device_type,
            host,
            username,
            password,
            port,
            timeout_seconds,
            secret,
            session_log,
        }
    }
}

// Internal function to convert PyDeviceConfig to DeviceConfig
fn py_config_to_rust_config(config: &PyDeviceConfig) -> DeviceConfig {
    DeviceConfig {
        device_type: config.device_type.clone(),
        host: config.host.clone(),
        username: config.username.clone(),
        password: config.password.clone(),
        port: config.port,
        timeout: config.timeout_seconds.map(Duration::from_secs),
        secret: config.secret.clone(),
        session_log: config.session_log.clone(),
    }
}

/// Python wrapper for NetworkDeviceConnection
///
/// This class represents a connection to a network device and provides
/// methods for sending commands, managing configuration mode, and more.
#[pyclass]
struct PyNetworkDevice {
    device: Box<dyn NetworkDeviceConnection + Send>,
}

#[pymethods]
impl PyNetworkDevice {
    /// Create a new device from config
    ///
    /// Creates a new network device connection handler based on the provided configuration.
    ///
    /// Args:
    ///     config: The device configuration
    ///
    /// Returns:
    ///     PyNetworkDevice: A new network device connection instance
    ///
    /// Raises:
    ///     RuntimeError: If device creation fails
    #[staticmethod]
    #[pyo3(signature = (config))]
    #[pyo3(text_signature = "(config)")]
    fn create(config: &PyDeviceConfig) -> PyResult<Self> {
        let rust_config = py_config_to_rust_config(config);
        let device =
            DeviceFactory::create_device(&rust_config).map_err(netssh_error_to_pyerr_simple)?;

        Ok(Self { device })
    }

    /// Connect to the device
    ///
    /// Establishes an SSH connection to the network device and performs initial setup.
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     ConnectionError: If connection fails
    ///     AuthenticationError: If authentication fails
    ///     TimeoutError: If connection times out
    #[pyo3(signature = ())]
    #[pyo3(text_signature = "()")]
    fn connect(&mut self) -> PyResult<()> {
        self.device.connect().map_err(netssh_error_to_pyerr_simple)
    }

    /// Close the connection to the device
    ///
    /// Returns:
    ///     None
    #[pyo3(signature = ())]
    #[pyo3(text_signature = "()")]
    fn close(&mut self) -> PyResult<()> {
        self.device.close().map_err(netssh_error_to_pyerr_simple)
    }

    /// Check if the device is in configuration mode
    ///
    /// Returns:
    ///     bool: True if device is in configuration mode, False otherwise
    #[pyo3(signature = ())]
    #[pyo3(text_signature = "()")]
    fn check_config_mode(&mut self) -> PyResult<bool> {
        self.device
            .check_config_mode()
            .map_err(netssh_error_to_pyerr_simple)
    }

    /// Enter configuration mode
    ///
    /// Args:
    ///     config_command (str, optional): Custom configuration command to use
    ///
    /// Returns:
    ///     CommandResult: Result of the command execution
    #[pyo3(signature = (config_command=None))]
    #[pyo3(text_signature = "(config_command=None)")]
    fn enter_config_mode(&mut self, config_command: Option<&str>) -> PyResult<PyCommandResult> {
        let start_time = Utc::now();
        let result = self.device.enter_config_mode(config_command);
        let end_time = Utc::now();

        match result {
            Ok(_) => {
                let device_type = self.device.get_device_type().to_string();
                let cmd = config_command.unwrap_or("configure terminal").to_string();
                let output_string = String::new();

                Ok(PyCommandResult::from(CommandResult::success(
                    get_device_hostname(&self.device),
                    device_type,
                    cmd,
                    output_string,
                    start_time,
                    end_time,
                )))
            }
            Err(err) => {
                let device_type = self.device.get_device_type().to_string();
                let cmd = config_command.unwrap_or("configure terminal").to_string();

                // Extract any output from the error and get the PyErr
                let (py_err, output_opt) = netssh_error_to_pyerr(err);

                // Extract error message from PyErr
                let error_text = py_err.to_string();

                // Use output from error if available
                let output = output_opt.unwrap_or_else(String::new);

                Ok(PyCommandResult::from(CommandResult::failure(
                    get_device_hostname(&self.device),
                    device_type,
                    cmd,
                    output,
                    start_time,
                    end_time,
                    error_text,
                )))
            }
        }
    }

    /// Exit configuration mode
    ///
    /// Args:
    ///     exit_command (str, optional): Custom exit command to use
    ///
    /// Returns:
    ///     CommandResult: Result of the command execution
    #[pyo3(signature = (exit_command=None))]
    #[pyo3(text_signature = "(exit_command=None)")]
    fn exit_config_mode(&mut self, exit_command: Option<&str>) -> PyResult<PyCommandResult> {
        let start_time = Utc::now();
        let result = self.device.exit_config_mode(exit_command);
        let end_time = Utc::now();

        match result {
            Ok(_) => {
                let device_type = self.device.get_device_type().to_string();
                let cmd = exit_command.unwrap_or("exit").to_string();
                let output_string = String::new();

                Ok(PyCommandResult::from(CommandResult::success(
                    get_device_hostname(&self.device),
                    device_type,
                    cmd,
                    output_string,
                    start_time,
                    end_time,
                )))
            }
            Err(err) => {
                let device_type = self.device.get_device_type().to_string();
                let cmd = exit_command.unwrap_or("exit").to_string();

                // Extract any output from the error and get the PyErr
                let (py_err, output_opt) = netssh_error_to_pyerr(err);

                // Extract error message from PyErr
                let error_text = py_err.to_string();

                // Use output from error if available
                let output = output_opt.unwrap_or_else(String::new);

                Ok(PyCommandResult::from(CommandResult::failure(
                    get_device_hostname(&self.device),
                    device_type,
                    cmd,
                    output,
                    start_time,
                    end_time,
                    error_text,
                )))
            }
        }
    }

    /// Prepare the session after connection
    ///
    /// Returns:
    ///     None
    #[pyo3(signature = ())]
    #[pyo3(text_signature = "()")]
    fn session_preparation(&mut self) -> PyResult<()> {
        self.device
            .session_preparation()
            .map_err(netssh_error_to_pyerr_simple)
    }

    /// Configure terminal settings
    ///
    /// Returns:
    ///     None
    #[pyo3(signature = ())]
    #[pyo3(text_signature = "()")]
    fn terminal_settings(&mut self) -> PyResult<()> {
        self.device
            .terminal_settings()
            .map_err(netssh_error_to_pyerr_simple)
    }

    /// Set terminal width
    ///
    /// Args:
    ///     width (int): Terminal width in characters
    ///
    /// Returns:
    ///     None
    #[pyo3(signature = (width))]
    #[pyo3(text_signature = "(width)")]
    fn set_terminal_width(&mut self, width: u32) -> PyResult<()> {
        self.device
            .set_terminal_width(width)
            .map_err(netssh_error_to_pyerr_simple)
    }

    /// Disable paging on the device
    ///
    /// Returns:
    ///     None
    #[pyo3(signature = ())]
    #[pyo3(text_signature = "()")]
    fn disable_paging(&mut self) -> PyResult<()> {
        self.device
            .disable_paging()
            .map_err(netssh_error_to_pyerr_simple)
    }

    /// Set the base prompt
    ///
    /// Returns:
    ///     str: The detected base prompt
    #[pyo3(signature = ())]
    #[pyo3(text_signature = "()")]
    fn set_base_prompt(&mut self) -> PyResult<String> {
        self.device
            .set_base_prompt()
            .map_err(netssh_error_to_pyerr_simple)
    }

    /// Save the device configuration
    ///
    /// Returns:
    ///     CommandResult: Result of the save configuration command
    #[pyo3(signature = ())]
    #[pyo3(text_signature = "()")]
    fn save_configuration(&mut self) -> PyResult<PyCommandResult> {
        let start_time = Utc::now();
        let result = self.device.save_configuration();
        let end_time = Utc::now();

        match result {
            Ok(_) => {
                let device_type = self.device.get_device_type().to_string();
                let cmd = "save configuration".to_string();
                let output_string = String::new();

                Ok(PyCommandResult::from(CommandResult::success(
                    get_device_hostname(&self.device),
                    device_type,
                    cmd,
                    output_string,
                    start_time,
                    end_time,
                )))
            }
            Err(err) => {
                let device_type = self.device.get_device_type().to_string();
                let cmd = "save configuration".to_string();

                // Extract any output from the error and get the PyErr
                let (py_err, output_opt) = netssh_error_to_pyerr(err);

                // Extract error message from PyErr
                let error_text = py_err.to_string();

                // Use output from error if available
                let output = output_opt.unwrap_or_else(String::new);

                Ok(PyCommandResult::from(CommandResult::failure(
                    get_device_hostname(&self.device),
                    device_type,
                    cmd,
                    output,
                    start_time,
                    end_time,
                    error_text,
                )))
            }
        }
    }

    /// Send a command to the device and return the output
    ///
    /// Args:
    ///     command (str): The command to execute on the device
    ///     expect_string (str, optional): Pattern to search for in output
    ///     read_timeout (float, optional): How long to wait for output in seconds (default: 10.0)
    ///     auto_find_prompt (bool, optional): Whether to automatically find prompt (default: True)
    ///     strip_prompt (bool, optional): Whether to strip prompt from output (default: True)
    ///     strip_command (bool, optional): Whether to strip command from output (default: True)
    ///     normalize (bool, optional): Whether to normalize line feeds (default: True)
    ///     cmd_verify (bool, optional): Whether to verify command echoing (default: True)
    ///
    /// Returns:
    ///     CommandResult: Result of the command execution containing output, timing information, and status
    #[pyo3(signature = (command, expect_string=None, read_timeout=None, auto_find_prompt=None, strip_prompt=None, strip_command=None, normalize=None, cmd_verify=None))]
    #[pyo3(
        text_signature = "(command, expect_string=None, read_timeout=None, auto_find_prompt=None, strip_prompt=None, strip_command=None, normalize=None, cmd_verify=None)"
    )]
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
    ) -> PyResult<PyCommandResult> {
        let start_time = Utc::now();
        let result = self.device.send_command(
            command,
            expect_string,
            read_timeout,
            auto_find_prompt,
            strip_prompt,
            strip_command,
            normalize,
            cmd_verify,
        );
        let end_time = Utc::now();

        match result {
            Ok(output) => {
                let device_type = self.device.get_device_type().to_string();
                let cmd = command.to_string();

                Ok(PyCommandResult::from(CommandResult::success(
                    get_device_hostname(&self.device),
                    device_type,
                    cmd,
                    output.to_string(),
                    start_time,
                    end_time,
                )))
            }
            Err(err) => {
                let device_type = self.device.get_device_type().to_string();
                let cmd = command.to_string();

                // Extract any output from the error and get the PyErr
                let (py_err, output_opt) = netssh_error_to_pyerr(err);

                // Extract error message from PyErr
                let error_text = py_err.to_string();

                // Use output from error if available
                let output = output_opt.unwrap_or_else(String::new);

                Ok(PyCommandResult::from(CommandResult::failure(
                    get_device_hostname(&self.device),
                    device_type,
                    cmd,
                    output,
                    start_time,
                    end_time,
                    error_text,
                )))
            }
        }
    }

    /// Send a command to the device and return the output (simple version)
    ///
    /// This is a simplified version that uses all default parameters for backward compatibility.
    ///
    /// Args:
    ///     command (str): The command to execute on the device
    ///
    /// Returns:
    ///     CommandResult: Result of the command execution
    #[pyo3(signature = (command))]
    #[pyo3(text_signature = "(command)")]
    fn send_command_simple(&mut self, command: &str) -> PyResult<PyCommandResult> {
        self.send_command(command, None, None, None, None, None, None, None)
    }

    /// Get the device type
    ///
    /// Returns:
    ///     str: The device type string
    #[pyo3(signature = ())]
    #[pyo3(text_signature = "()")]
    fn get_device_type(&self) -> &str {
        self.device.get_device_type()
    }

    /// Context manager support - enter
    #[pyo3(signature = ())]
    #[pyo3(text_signature = "()")]
    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    /// Context manager support - exit
    #[pyo3(signature = (*, _exc_type=None, _exc_value=None, _traceback=None))]
    fn __exit__(
        &mut self,
        _exc_type: Option<&PyAny>,
        _exc_value: Option<&PyAny>,
        _traceback: Option<&PyAny>,
    ) -> PyResult<bool> {
        self.close()?;
        Ok(false)
    }

    /// Send configuration commands to the device
    ///
    /// Args:
    ///     config_commands (list): List of configuration commands to send
    ///     exit_config_mode (bool, optional): Whether to exit config mode after sending commands
    ///     read_timeout (float, optional): Read timeout in seconds
    ///     strip_prompt (bool, optional): Whether to strip the prompt from the output
    ///     strip_command (bool, optional): Whether to strip the command from the output
    ///     config_mode_command (str, optional): Custom command to enter config mode
    ///     cmd_verify (bool, optional): Whether to verify command echoing
    ///     enter_config_mode (bool, optional): Whether to enter config mode before sending commands
    ///     error_pattern (str, optional): Regex pattern to detect errors in output
    ///     terminator (str, optional): Command terminator character
    ///     bypass_commands (str, optional): Commands to bypass verification
    ///     fast_cli (bool, optional): Whether to optimize for faster command execution
    ///
    /// Returns:
    ///     CommandResult: Result of the config commands execution
    #[pyo3(signature = (
        config_commands,
        exit_config_mode=None,
        read_timeout=None,
        strip_prompt=None,
        strip_command=None,
        config_mode_command=None,
        cmd_verify=None,
        enter_config_mode=None,
        error_pattern=None,
        terminator=None,
        bypass_commands=None,
        fast_cli=None,
    ))]
    #[pyo3(
        text_signature = "(config_commands, exit_config_mode=None, read_timeout=None, strip_prompt=None, strip_command=None, config_mode_command=None, cmd_verify=None, enter_config_mode=None, error_pattern=None, terminator=None, bypass_commands=None, fast_cli=None)"
    )]
    fn send_config_set(
        &mut self,
        config_commands: &PyList,
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
    ) -> PyResult<PyCommandResult> {
        let commands: Vec<String> = config_commands
            .iter()
            .map(|item| item.extract::<String>())
            .collect::<Result<Vec<String>, _>>()?;

        let start_time = Utc::now();
        let result = self.device.send_config_set(
            commands,
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
        );
        let end_time = Utc::now();

        match result {
            Ok(output) => {
                let device_type = self.device.get_device_type().to_string();
                let cmd = "config set".to_string();

                Ok(PyCommandResult::from(CommandResult::success(
                    get_device_hostname(&self.device),
                    device_type,
                    cmd,
                    output.to_string(),
                    start_time,
                    end_time,
                )))
            }
            Err(err) => {
                let device_type = self.device.get_device_type().to_string();
                let cmd = "config set".to_string();

                // Extract any output from the error and get the PyErr
                let (py_err, output_opt) = netssh_error_to_pyerr(err);

                // Extract error message from PyErr
                let error_text = py_err.to_string();

                // Use output from error if available
                let output = output_opt.unwrap_or_else(String::new);

                Ok(PyCommandResult::from(CommandResult::failure(
                    get_device_hostname(&self.device),
                    device_type,
                    cmd,
                    output,
                    start_time,
                    end_time,
                    error_text,
                )))
            }
        }
    }
}

/// Python wrapper for CommandResult
///
/// This class represents the result of executing a command on a network device.
/// It contains detailed information about the command execution, including:
/// - device_id: The identifier for the device
/// - device_type: The type of device
/// - command: The command that was executed
/// - output: The output text from the command
/// - start_time: When command execution started
/// - end_time: When command execution ended
/// - duration_ms: How long the command took to execute in milliseconds
/// - status: The execution status (Success, Failed, Timeout, Skipped)
/// - error: Error message if the command failed
#[pyclass]
#[derive(Clone)]
struct PyCommandResult {
    #[pyo3(get)]
    device_id: String,
    #[pyo3(get)]
    device_type: String,
    #[pyo3(get)]
    command: String,
    #[pyo3(get)]
    output: Option<String>,
    #[pyo3(get)]
    start_time: String,
    #[pyo3(get)]
    end_time: String,
    #[pyo3(get)]
    duration_ms: u64,
    #[pyo3(get)]
    status: String,
    #[pyo3(get)]
    error: Option<String>,
}

impl From<CommandResult> for PyCommandResult {
    fn from(result: CommandResult) -> Self {
        Self {
            device_id: result.device_id,
            device_type: result.device_type,
            command: result.command,
            output: result.output,
            start_time: result.start_time.to_rfc3339(),
            end_time: result.end_time.to_rfc3339(),
            duration_ms: result.duration_ms,
            status: format!("{:?}", result.status),
            error: result.error,
        }
    }
}

impl ToPyObject for PyCommandResult {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        // We need to create a new PyCell with our PyCommandResult instance
        PyCell::new(py, self.clone()).unwrap().into()
    }
}

#[pymethods]
impl PyCommandResult {
    /// Convert the command result to a Python dictionary
    ///
    /// Returns:
    ///     dict: Dictionary representation of the command result
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        let dict = PyDict::new(py);
        dict.set_item("device_id", &self.device_id)?;
        dict.set_item("device_type", &self.device_type)?;
        dict.set_item("command", &self.command)?;
        dict.set_item("output", &self.output)?;
        dict.set_item("start_time", &self.start_time)?;
        dict.set_item("end_time", &self.end_time)?;
        dict.set_item("duration_ms", &self.duration_ms)?;
        dict.set_item("status", &self.status)?;
        dict.set_item("error", &self.error)?;
        Ok(dict)
    }

    /// String representation of the command result
    ///
    /// Returns:
    ///     str: A formatted string describing the command result
    fn __str__(&self) -> PyResult<String> {
        Ok(format!(
            "Command '{}' on device '{}' ({}): Status={}, Duration={}ms",
            self.command, self.device_id, self.device_type, self.status, self.duration_ms
        ))
    }

    /// Check if the command was successful
    ///
    /// Returns:
    ///     bool: True if command executed successfully, False otherwise
    fn is_success(&self) -> bool {
        self.status == "Success"
    }

    /// Check if the command failed
    ///
    /// Returns:
    ///     bool: True if command failed, False otherwise
    fn is_failure(&self) -> bool {
        self.status == "Failed"
    }

    /// Check if the command timed out
    ///
    /// Returns:
    ///     bool: True if command timed out, False otherwise
    fn is_timeout(&self) -> bool {
        self.status == "Timeout"
    }
}

/// Python wrapper for BatchCommandResults
///
/// This class represents the results of executing commands on multiple devices.
/// It provides methods to access and analyze the results in various formats.
#[pyclass]
struct PyBatchCommandResults {
    results: BatchCommandResults,
}

impl From<BatchCommandResults> for PyBatchCommandResults {
    fn from(results: BatchCommandResults) -> Self {
        Self { results }
    }
}

#[pymethods]
impl PyBatchCommandResults {
    /// Get all results for a specific device
    ///
    /// Args:
    ///     device_id (str): The device identifier
    ///
    /// Returns:
    ///     list[CommandResult]: A list of CommandResult objects for the specified device, or None if device not found
    fn get_device_results<'py>(
        &self,
        py: Python<'py>,
        device_id: &str,
    ) -> PyResult<Option<&'py PyList>> {
        match self.results.get_device_results(device_id) {
            Some(device_results) => {
                // Convert results to PyList of PyCommandResult objects
                let py_list = PyList::empty(py);
                for result in device_results {
                    let py_result = PyCommandResult::from(result.clone());
                    py_list.append(py_result)?;
                }
                Ok(Some(py_list))
            }
            None => Ok(None),
        }
    }

    /// Get all command results across all devices
    ///
    /// Returns:
    ///     list[CommandResult]: A list of all CommandResult objects
    fn get_all_results<'py>(&self, py: Python<'py>) -> PyResult<&'py PyList> {
        let py_list = PyList::empty(py);
        for results in self.results.results.values() {
            for result in results {
                let py_result = PyCommandResult::from(result.clone());
                py_list.append(py_result)?;
            }
        }
        Ok(py_list)
    }

    /// Get all successful command results
    ///
    /// Returns:
    ///     list[CommandResult]: A list of CommandResult objects with Success status
    fn get_successful_results<'py>(&self, py: Python<'py>) -> PyResult<&'py PyList> {
        let py_list = PyList::empty(py);
        for result in self.results.successful_results() {
            let py_result = PyCommandResult::from(result.clone());
            py_list.append(py_result)?;
        }
        Ok(py_list)
    }

    /// Get all failed command results
    ///
    /// Returns:
    ///     list[CommandResult]: A list of CommandResult objects with Failed status
    fn get_failed_results<'py>(&self, py: Python<'py>) -> PyResult<&'py PyList> {
        let py_list = PyList::empty(py);
        for result in self.results.failed_results() {
            let py_result = PyCommandResult::from(result.clone());
            py_list.append(py_result)?;
        }
        Ok(py_list)
    }

    /// Get results for a specific command across all devices
    ///
    /// Args:
    ///     command (str): The command to filter by
    ///
    /// Returns:
    ///     list[CommandResult]: A list of CommandResult objects for the specified command
    fn get_command_results<'py>(&self, py: Python<'py>, command: &str) -> PyResult<&'py PyList> {
        // Use the get_command_results method from BatchCommandResults
        let py_list = PyList::empty(py);
        for result in self.results.get_command_results(command) {
            let py_result = PyCommandResult::from(result.clone());
            py_list.append(py_result)?;
        }
        Ok(py_list)
    }

    /// Format the results as a table for display
    ///
    /// Returns:
    ///     str: A formatted string containing a table of results
    fn format_as_table(&self) -> String {
        format!("{:#?}", self.results)
    }

    /// Convert the batch results to JSON
    ///
    /// Returns:
    ///     str: A JSON string representation of the results
    fn to_json(&self) -> PyResult<String> {
        serde_json::to_string_pretty(&self.results)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Convert the batch results to CSV
    ///
    /// Returns:
    ///     str: A CSV string representation of the results
    fn to_csv(&self) -> String {
        let mut csv = String::from(
            "device_id,device_type,command,status,duration_ms,start_time,end_time,error\n",
        );

        for results in self.results.results.values() {
            for result in results {
                let error_text = match &result.error {
                    Some(err) => err.replace(',', ";").replace('\n', " "),
                    None => String::from(""),
                };

                csv.push_str(&format!(
                    "{},{},{},{:?},{},{},{},{}\n",
                    result.device_id,
                    result.device_type,
                    result.command,
                    result.status,
                    result.duration_ms,
                    result.start_time,
                    result.end_time,
                    error_text
                ));
            }
        }

        csv
    }

    /// Compare command outputs across devices
    ///
    /// Args:
    ///     command (str): The command to compare across devices
    ///
    /// Returns:
    ///     dict: A dictionary mapping devices to their command outputs
    fn compare_outputs<'py>(&self, py: Python<'py>, command: &str) -> PyResult<&'py PyDict> {
        let dict = PyDict::new(py);

        // Get all results for the specified command
        let command_results = self.results.get_command_results(command);

        // Group by device and get the output
        for result in command_results {
            if let Some(output) = &result.output {
                dict.set_item(&result.device_id, output)?;
            }
        }

        Ok(dict)
    }

    /// Get the total number of commands executed
    ///
    /// Returns:
    ///     int: Total command count
    #[getter]
    fn command_count(&self) -> usize {
        self.results.command_count
    }

    /// Get the number of successful commands
    ///
    /// Returns:
    ///     int: Successful command count
    #[getter]
    fn success_count(&self) -> usize {
        self.results.success_count
    }

    /// Get the number of failed commands
    ///
    /// Returns:
    ///     int: Failed command count
    #[getter]
    fn failure_count(&self) -> usize {
        self.results.failure_count
    }

    /// Get the number of devices processed
    ///
    /// Returns:
    ///     int: Device count
    #[getter]
    fn device_count(&self) -> usize {
        self.results.device_count
    }

    /// Get the total duration of the batch execution in milliseconds
    ///
    /// Returns:
    ///     int: Duration in milliseconds
    #[getter]
    fn duration_ms(&self) -> u64 {
        self.results.duration_ms
    }

    /// String representation of the batch results
    ///
    /// Returns:
    ///     str: A formatted string describing the batch results
    fn __str__(&self) -> String {
        format!(
            "BatchCommandResults: {} devices, {} commands ({} success, {} failed), {}ms",
            self.results.device_count,
            self.results.command_count,
            self.results.success_count,
            self.results.failure_count,
            self.results.duration_ms
        )
    }
}

/// Python wrapper for ParallelExecutionManager
#[pyclass]
struct PyParallelExecutionManager {
    manager: ParallelExecutionManager,
}

#[pymethods]
impl PyParallelExecutionManager {
    /// Create a new parallel execution manager
    #[new]
    #[pyo3(signature = (max_concurrency=None, command_timeout_seconds=None, _connection_timeout_seconds=None, failure_strategy=None, reuse_connections=None))]
    #[pyo3(
        text_signature = "(max_concurrency=None, command_timeout_seconds=None, connection_timeout_seconds=None, failure_strategy=None, reuse_connections=None)"
    )]
    fn new(
        max_concurrency: Option<usize>,
        command_timeout_seconds: Option<u64>,
        _connection_timeout_seconds: Option<u64>,
        failure_strategy: Option<&str>,
        reuse_connections: Option<bool>,
    ) -> PyResult<Self> {
        // Start with default config
        let mut config = ParallelExecutionConfig::default();

        // Apply custom configuration if provided
        if let Some(max_concurrency) = max_concurrency {
            config.max_concurrent_devices = max_concurrency;
        }

        if let Some(timeout) = command_timeout_seconds {
            config.command_timeout = Duration::from_secs(timeout);
        }

        // Deprecated: connection_timeout is not used anymore
        // We just ignore it for backwards compatibility

        // Set failure strategy if provided
        if let Some(strategy) = failure_strategy {
            // Convert failure strategy string to enum
            let failure_strategy = match strategy.to_lowercase().as_str() {
                "continue_on_device" => FailureStrategy::ContinueOnDevice,
                "skip_device" => FailureStrategy::SkipDevice,
                "abort_batch" => FailureStrategy::AbortBatch,
                _ => FailureStrategy::ContinueOnDevice, // Default
            };

            config.failure_strategy = failure_strategy;
        }

        // Set reuse connections if provided
        if let Some(reuse) = reuse_connections {
            config.stop_on_first_failure = !reuse;
        }

        // Create manager with the configured settings
        let manager = ParallelExecutionManager::with_config(config);

        Ok(Self { manager })
    }

    /// Set the maximum concurrency
    #[pyo3(signature = (max_concurrency))]
    #[pyo3(text_signature = "(max_concurrency)")]
    fn set_max_concurrency(&mut self, max_concurrency: usize) {
        self.manager.set_max_concurrency(max_concurrency);
    }

    /// Set the command timeout
    #[pyo3(signature = (timeout_seconds))]
    #[pyo3(text_signature = "(timeout_seconds)")]
    fn set_command_timeout(&mut self, timeout_seconds: u64) {
        self.manager
            .set_command_timeout(Duration::from_secs(timeout_seconds));
    }

    /// Set the connection timeout (deprecated)
    #[pyo3(signature = (_timeout_seconds))]
    #[pyo3(text_signature = "(timeout_seconds)")]
    fn set_connection_timeout(&mut self, _timeout_seconds: u64) {
        // This is now a no-op as connection_timeout is no longer used
        // We keep the method for backwards compatibility
    }

    /// Set failure strategy
    #[pyo3(signature = (strategy))]
    #[pyo3(text_signature = "(strategy)")]
    fn set_failure_strategy(&mut self, strategy: &str) {
        let failure_strategy = match strategy.to_lowercase().as_str() {
            "continue_on_device" => FailureStrategy::ContinueOnDevice,
            "skip_device" => FailureStrategy::SkipDevice,
            "abort_batch" => FailureStrategy::AbortBatch,
            _ => FailureStrategy::ContinueOnDevice, // Default
        };
        self.manager.set_failure_strategy(failure_strategy);
    }

    /// Set whether to reuse connections
    #[pyo3(signature = (reuse))]
    #[pyo3(text_signature = "(reuse)")]
    fn set_reuse_connections(&mut self, reuse: bool) {
        // Note: stop_on_first_failure is the opposite of reuse_connections
        self.manager.set_reuse_connections(reuse);
    }

    /// Execute a command on all devices
    #[pyo3(signature = (configs, command))]
    #[pyo3(text_signature = "(configs, command)")]
    fn execute_command_on_all<'py>(
        &mut self,
        py: Python<'py>,
        configs: &PyList,
        command: &str,
    ) -> PyResult<PyBatchCommandResults> {
        // Extract Rust configs from Python configs outside the allow_threads block
        let rust_configs = extract_device_configs(configs)?;
        let command = command.to_string();

        py.allow_threads(|| {
            // Create a future to execute command on all devices
            let future = self.manager.execute_command_on_all(rust_configs, command);

            // Execute the future in a tokio runtime
            let rt = tokio::runtime::Runtime::new().unwrap();
            let results =
                rt.block_on(async { future.await.map_err(netssh_error_to_pyerr_simple) })?;

            // Return the results wrapped in a PyBatchCommandResults
            Ok(PyBatchCommandResults { results })
        })
    }

    /// Execute multiple commands on all devices
    #[pyo3(signature = (configs, commands))]
    #[pyo3(text_signature = "(configs, commands)")]
    fn execute_commands_on_all<'py>(
        &mut self,
        py: Python<'py>,
        configs: &PyList,
        commands: Vec<String>,
    ) -> PyResult<PyBatchCommandResults> {
        // Extract Rust configs from Python configs outside the allow_threads block
        let rust_configs = extract_device_configs(configs)?;
        let commands_clone = commands.clone();

        py.allow_threads(|| {
            // Create a future to execute commands on all devices
            let future = self
                .manager
                .execute_commands_on_all(rust_configs, commands_clone);

            // Execute the future in a tokio runtime
            let rt = tokio::runtime::Runtime::new().unwrap();
            let results =
                rt.block_on(async { future.await.map_err(netssh_error_to_pyerr_simple) })?;

            // Return the results wrapped in a PyBatchCommandResults
            Ok(PyBatchCommandResults { results })
        })
    }

    /// Execute commands on specific devices
    #[pyo3(signature = (device_commands))]
    #[pyo3(text_signature = "(device_commands)")]
    fn execute_commands(
        &mut self,
        py: Python<'_>,
        device_commands: &PyDict,
    ) -> PyResult<PyBatchCommandResults> {
        // Extract device commands map outside the allow_threads block
        let mut device_map = HashMap::new();

        for (k, v) in device_commands.iter() {
            // Get the PyDeviceConfig
            let config = match k.extract::<&PyCell<PyDeviceConfig>>() {
                Ok(cell) => {
                    // Extract the PyDeviceConfig from the cell
                    let config_ref = cell.try_borrow().map_err(|_| {
                        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                            "Failed to borrow PyDeviceConfig",
                        )
                    })?;

                    py_config_to_rust_config(&config_ref)
                }
                Err(_) => {
                    return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                        "Keys must be PyDeviceConfig objects",
                    ));
                }
            };

            // Get the commands - could be a string or list of strings
            let commands = if let Ok(cmd) = v.extract::<String>() {
                vec![cmd]
            } else if let Ok(cmds) = v.extract::<Vec<String>>() {
                cmds
            } else {
                return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                    "Values must be strings or lists of strings",
                ));
            };

            device_map.insert(config, commands);
        }

        py.allow_threads(move || {
            // Create a future to execute commands
            let future = self.manager.execute_commands(device_map);

            // Execute the future in a tokio runtime
            let rt = tokio::runtime::Runtime::new().unwrap();
            let results =
                rt.block_on(async { future.await.map_err(netssh_error_to_pyerr_simple) })?;

            // Return the results wrapped in a PyBatchCommandResults
            Ok(PyBatchCommandResults { results })
        })
    }

    /// Close all open connections
    #[pyo3(signature = ())]
    #[pyo3(text_signature = "()")]
    fn cleanup(&mut self) {
        self.manager.cleanup();
    }

    /// Context manager support - enter
    #[pyo3(signature = ())]
    #[pyo3(text_signature = "()")]
    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    /// Context manager support - exit
    #[pyo3(signature = (*, _exc_type=None, _exc_value=None, _traceback=None))]
    fn __exit__(
        &mut self,
        _exc_type: Option<&PyAny>,
        _exc_value: Option<&PyAny>,
        _traceback: Option<&PyAny>,
    ) -> PyResult<bool> {
        self.cleanup();
        Ok(false) // Don't suppress exceptions
    }
}

// Helper function to extract DeviceConfig objects from a PyList
fn extract_device_configs(configs: &PyList) -> PyResult<Vec<DeviceConfig>> {
    let mut rust_configs = Vec::new();

    for item in configs.iter() {
        if let Ok(py_config) = item.extract::<PyDeviceConfig>() {
            rust_configs.push(py_config_to_rust_config(&py_config));
        } else {
            return Err(PyRuntimeError::new_err(
                "List must contain only DeviceConfig objects",
            ));
        }
    }

    Ok(rust_configs)
}

// Add a helper function for getting hostname with a default value
fn get_device_hostname(device: &Box<dyn NetworkDeviceConnection + Send>) -> String {
    // Use device_type or other identifier when hostname is not available
    device.get_device_type().to_string()
}
