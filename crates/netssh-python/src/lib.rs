use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pyo3::wrap_pyfunction;
use std::collections::HashMap;
use std::time::Duration;

use netssh_core::device_connection::{DeviceConfig, DeviceInfo, NetworkDeviceConnection};
use netssh_core::device_factory::DeviceFactory;
use netssh_core::error::NetsshError;
use netssh_core::parallel_execution::{
    BatchCommandResults, CommandResult, FailureStrategy, ParallelExecutionConfig,
    ParallelExecutionManager,
};

/// Python module for netssh-rs
#[pymodule]
fn netssh_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyDeviceConfig>()?;
    m.add_class::<PyDeviceInfo>()?;
    m.add_class::<PyNetworkDevice>()?;
    m.add_class::<PyCommandResult>()?;
    m.add_class::<PyBatchCommandResults>()?;
    m.add_class::<PyParallelExecutionManager>()?;
    m.add_function(wrap_pyfunction!(initialize_logging, m)?)?;

    Ok(())
}

/// Convert NetsshError to PyErr
fn netssh_error_to_pyerr(err: NetsshError) -> PyErr {
    PyRuntimeError::new_err(format!("{}", err))
}

/// Initialize logging
#[pyfunction]
#[pyo3(signature = (debug=false, console=false))]
#[pyo3(text_signature = "(debug=False, console=False)")]
fn initialize_logging(debug: bool, console: bool) -> PyResult<()> {
    netssh_core::logging::init_logging(debug, console).map_err(netssh_error_to_pyerr)
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

/// Python wrapper for DeviceInfo
#[pyclass]
struct PyDeviceInfo {
    #[pyo3(get)]
    device_type: String,
    #[pyo3(get)]
    model: String,
    #[pyo3(get)]
    version: String,
    #[pyo3(get)]
    hostname: String,
    #[pyo3(get)]
    serial: String,
    #[pyo3(get)]
    uptime: String,
}

impl From<DeviceInfo> for PyDeviceInfo {
    fn from(info: DeviceInfo) -> Self {
        Self {
            device_type: info.device_type,
            model: info.model,
            version: info.version,
            hostname: info.hostname,
            serial: info.serial,
            uptime: info.uptime,
        }
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
    ///     A new PyNetworkDevice instance
    ///
    /// Raises:
    ///     RuntimeError: If device creation fails
    #[staticmethod]
    #[pyo3(signature = (config))]
    #[pyo3(text_signature = "(config)")]
    fn create(config: &PyDeviceConfig) -> PyResult<Self> {
        let rust_config = py_config_to_rust_config(config);
        let device = DeviceFactory::create_device(&rust_config).map_err(netssh_error_to_pyerr)?;

        Ok(Self { device })
    }

    /// Connect to the device
    ///
    /// Establishes an SSH connection to the network device and performs initial setup.
    ///
    /// Raises:
    ///     ConnectionError: If connection fails
    ///     AuthenticationError: If authentication fails
    ///     TimeoutError: If connection times out
    #[pyo3(signature = ())]
    #[pyo3(text_signature = "()")]
    fn connect(&mut self) -> PyResult<()> {
        self.device.connect().map_err(netssh_error_to_pyerr)
    }

    /// Close the connection to the device
    #[pyo3(signature = ())]
    #[pyo3(text_signature = "()")]
    fn close(&mut self) -> PyResult<()> {
        self.device.close().map_err(netssh_error_to_pyerr)
    }

    /// Check if the device is in configuration mode
    #[pyo3(signature = ())]
    #[pyo3(text_signature = "()")]
    fn check_config_mode(&mut self) -> PyResult<bool> {
        self.device
            .check_config_mode()
            .map_err(netssh_error_to_pyerr)
    }

    /// Enter configuration mode
    #[pyo3(signature = (config_command=None))]
    #[pyo3(text_signature = "(config_command=None)")]
    fn enter_config_mode(&mut self, config_command: Option<&str>) -> PyResult<()> {
        self.device
            .enter_config_mode(config_command)
            .map_err(netssh_error_to_pyerr)
    }

    /// Exit configuration mode
    #[pyo3(signature = (exit_command=None))]
    #[pyo3(text_signature = "(exit_command=None)")]
    fn exit_config_mode(&mut self, exit_command: Option<&str>) -> PyResult<()> {
        self.device
            .exit_config_mode(exit_command)
            .map_err(netssh_error_to_pyerr)
    }

    /// Prepare the session after connection
    #[pyo3(signature = ())]
    #[pyo3(text_signature = "()")]
    fn session_preparation(&mut self) -> PyResult<()> {
        self.device
            .session_preparation()
            .map_err(netssh_error_to_pyerr)
    }

    /// Configure terminal settings
    #[pyo3(signature = ())]
    #[pyo3(text_signature = "()")]
    fn terminal_settings(&mut self) -> PyResult<()> {
        self.device
            .terminal_settings()
            .map_err(netssh_error_to_pyerr)
    }

    /// Set terminal width
    #[pyo3(signature = (width))]
    #[pyo3(text_signature = "(width)")]
    fn set_terminal_width(&mut self, width: u32) -> PyResult<()> {
        self.device
            .set_terminal_width(width)
            .map_err(netssh_error_to_pyerr)
    }

    /// Disable paging
    #[pyo3(signature = ())]
    #[pyo3(text_signature = "()")]
    fn disable_paging(&mut self) -> PyResult<()> {
        self.device.disable_paging().map_err(netssh_error_to_pyerr)
    }

    /// Set base prompt
    #[pyo3(signature = ())]
    #[pyo3(text_signature = "()")]
    fn set_base_prompt(&mut self) -> PyResult<String> {
        self.device.set_base_prompt().map_err(netssh_error_to_pyerr)
    }

    /// Save or commit configuration
    #[pyo3(signature = ())]
    #[pyo3(text_signature = "()")]
    fn save_configuration(&mut self) -> PyResult<()> {
        self.device
            .save_configuration()
            .map_err(netssh_error_to_pyerr)
    }

    /// Send command to device
    ///
    /// Sends a command to the device and returns the output.
    ///
    /// Args:
    ///     command: The command to execute
    ///
    /// Returns:
    ///     The command output as a string
    ///
    /// Raises:
    ///     ConnectionError: If the device is not connected
    ///     RuntimeError: If the command execution fails
    ///     TimeoutError: If the command times out
    #[pyo3(signature = (command))]
    #[pyo3(text_signature = "(command)")]
    fn send_command(&mut self, command: &str) -> PyResult<String> {
        self.device
            .send_command(command)
            .map_err(netssh_error_to_pyerr)
    }

    /// Get the device type (vendor and model)
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
        Ok(false) // Don't suppress exceptions
    }

    /// Send configuration commands to the device
    ///
    /// This method sends multiple configuration commands to the device. It
    /// handles entering and exiting config mode as well as various options
    /// for command verification and error detection.
    ///
    /// Args:
    ///     config_commands: A list of configuration commands to send
    ///     exit_config_mode: Whether to exit config mode after sending commands (default: True)
    ///     read_timeout: Timeout for reading output after sending commands (in seconds, default: 15.0)
    ///     strip_prompt: Whether to strip the prompt from output (default: False)
    ///     strip_command: Whether to strip command echo from output (default: False)
    ///     config_mode_command: Custom command to enter config mode (optional)
    ///     cmd_verify: Whether to verify command echoes (default: True)
    ///     enter_config_mode: Whether to enter config mode before sending commands (default: True)
    ///     error_pattern: Regex pattern to detect command errors (optional)
    ///     terminator: Alternate terminator pattern to detect end of output (optional)
    ///     bypass_commands: Regex pattern for commands that should bypass verification (optional)
    ///     fast_cli: Whether to use fast mode with minimal verification (default: False)
    ///
    /// Returns:
    ///     The output from the configuration commands
    ///
    /// Raises:
    ///     RuntimeError: If an error occurs during configuration
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
        fast_cli=None
    ))]
    #[pyo3(
        text_signature = "(config_commands, exit_config_mode=True, read_timeout=15.0, strip_prompt=False, strip_command=False, config_mode_command=None, cmd_verify=True, enter_config_mode=True, error_pattern=None, terminator=None, bypass_commands=None, fast_cli=False)"
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
    ) -> PyResult<String> {
        // Convert the Python list of commands to a Vec of Strings
        let commands: Vec<String> = config_commands
            .iter()
            .map(|x| x.extract::<String>())
            .collect::<Result<Vec<String>, _>>()?;

        // Call the Rust implementation
        self.device
            .send_config_set(
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
            )
            .map_err(netssh_error_to_pyerr)
    }
}

/// Python wrapper for CommandResult
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
            start_time: result.start_time.to_string(),
            end_time: result.end_time.to_string(),
            duration_ms: result.duration_ms,
            status: format!("{:?}", result.status),
            error: result.error,
        }
    }
}

#[pymethods]
impl PyCommandResult {
    /// Convert to Python dictionary
    #[pyo3(signature = ())]
    #[pyo3(text_signature = "()")]
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        let dict = PyDict::new(py);
        dict.set_item("device_id", &self.device_id)?;
        dict.set_item("device_type", &self.device_type)?;
        dict.set_item("command", &self.command)?;
        dict.set_item("output", &self.output)?;
        dict.set_item("start_time", &self.start_time)?;
        dict.set_item("end_time", &self.end_time)?;
        dict.set_item("duration_ms", self.duration_ms)?;
        dict.set_item("status", &self.status)?;
        dict.set_item("error", &self.error)?;
        Ok(dict)
    }
}

/// Python wrapper for BatchCommandResults
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
    #[pyo3(signature = (device_id))]
    #[pyo3(text_signature = "(device_id)")]
    fn get_device_results<'py>(
        &self,
        py: Python<'py>,
        device_id: &str,
    ) -> PyResult<Option<&'py PyList>> {
        // Get the results for a specific device
        let results = self.results.get_device_results(device_id);

        // If no results for this device, return None
        if let Some(results) = results {
            if results.is_empty() {
                return Ok(None);
            }
        } else {
            return Ok(None);
        }

        // Convert results to PyList of PyCommandResult objects
        let py_list = PyList::empty(py);

        if let Some(results) = results {
            for result in results {
                let py_result = PyCommandResult::from(result.clone());
                py_list.append(py_result.to_dict(py)?)?;
            }
        }

        Ok(Some(py_list))
    }

    /// Get all results
    #[pyo3(signature = ())]
    #[pyo3(text_signature = "()")]
    fn get_all_results<'py>(&self, py: Python<'py>) -> PyResult<&'py PyList> {
        let py_list = PyList::empty(py);

        // Collect all results from all devices
        for (_, device_results) in self.results.results.iter() {
            for result in device_results {
                let py_result = PyCommandResult::from(result.clone());
                py_list.append(py_result.to_dict(py)?)?;
            }
        }

        Ok(py_list)
    }

    /// Get successful results
    #[pyo3(signature = ())]
    #[pyo3(text_signature = "()")]
    fn get_successful_results<'py>(&self, py: Python<'py>) -> PyResult<&'py PyList> {
        let py_list = PyList::empty(py);

        // Collect all successful results from all devices
        for (_, device_results) in self.results.results.iter() {
            for result in device_results {
                if result.status == netssh_core::parallel_execution::CommandStatus::Success {
                    let py_result = PyCommandResult::from(result.clone());
                    py_list.append(py_result.to_dict(py)?)?;
                }
            }
        }

        Ok(py_list)
    }

    /// Get failed results
    #[pyo3(signature = ())]
    #[pyo3(text_signature = "()")]
    fn get_failed_results<'py>(&self, py: Python<'py>) -> PyResult<&'py PyList> {
        let py_list = PyList::empty(py);

        // Collect all failed results from all devices
        for (_, device_results) in self.results.results.iter() {
            for result in device_results {
                if result.status == netssh_core::parallel_execution::CommandStatus::Failed {
                    let py_result = PyCommandResult::from(result.clone());
                    py_list.append(py_result.to_dict(py)?)?;
                }
            }
        }

        Ok(py_list)
    }

    /// Get results for a specific command
    #[pyo3(signature = (command))]
    #[pyo3(text_signature = "(command)")]
    fn get_command_results<'py>(&self, py: Python<'py>, command: &str) -> PyResult<&'py PyList> {
        let py_list = PyList::empty(py);

        // Use the get_command_results method from BatchCommandResults
        let results = self.results.get_command_results(command);

        for result in results {
            let py_result = PyCommandResult::from(result.clone());
            py_list.append(py_result.to_dict(py)?)?;
        }

        Ok(py_list)
    }

    /// Format results as an ASCII table
    #[pyo3(signature = ())]
    #[pyo3(text_signature = "()")]
    fn format_as_table(&self) -> String {
        netssh_core::parallel_execution::utils::format_as_table(&self.results)
    }

    /// Convert results to JSON format
    #[pyo3(signature = ())]
    #[pyo3(text_signature = "()")]
    fn to_json(&self) -> PyResult<String> {
        netssh_core::parallel_execution::utils::to_json(&self.results)
            .map_err(|e| PyRuntimeError::new_err(format!("JSON serialization error: {}", e)))
    }

    /// Convert results to CSV format
    #[pyo3(signature = ())]
    #[pyo3(text_signature = "()")]
    fn to_csv(&self) -> String {
        netssh_core::parallel_execution::utils::to_csv(&self.results)
    }

    /// Compare outputs for the same command across devices
    #[pyo3(signature = (command))]
    #[pyo3(text_signature = "(command)")]
    fn compare_outputs<'py>(&self, py: Python<'py>, command: &str) -> PyResult<&'py PyDict> {
        // Use the compare_outputs function from the utilities module
        let comparisons =
            netssh_core::parallel_execution::utils::compare_outputs(&self.results, command);

        // Convert to Python dict of {device_id: {'unique': [...], 'common': [...]}}
        let py_dict = PyDict::new(py);

        for (device_id, comparison) in comparisons {
            let device_dict = PyDict::new(py);

            let unique_list = PyList::empty(py);
            for unique_line in comparison {
                unique_list.append(unique_line)?;
            }
            device_dict.set_item("unique", unique_list)?;

            // Common lines are not provided in this implementation
            let common_list = PyList::empty(py);
            device_dict.set_item("common", common_list)?;

            py_dict.set_item(device_id, device_dict)?;
        }

        Ok(py_dict)
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
            let results = rt.block_on(async { future.await.map_err(netssh_error_to_pyerr) })?;

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
            let results = rt.block_on(async { future.await.map_err(netssh_error_to_pyerr) })?;

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
            let results = rt.block_on(async { future.await.map_err(netssh_error_to_pyerr) })?;

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
