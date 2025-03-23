use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use pyo3::exceptions::PyRuntimeError;
use pyo3::types::{PyDict, PyList};
use std::collections::HashMap;
use std::time::Duration;

use netssh_core::device_connection::{NetworkDeviceConnection, DeviceConfig, DeviceInfo};
use netssh_core::device_factory::DeviceFactory;
use netssh_core::error::NetsshError;
use netssh_core::parallel_execution::{
    CommandResult, ParallelExecutionManager, 
    BatchCommandResults, ParallelExecutionConfig, FailureStrategy
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
fn initialize_logging(debug: bool, console: bool) -> PyResult<()> {
    netssh_core::logging::init_logging(debug, console)
        .map_err(netssh_error_to_pyerr)
}

/// Python wrapper for DeviceConfig
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
    vendor: String,
    #[pyo3(get)]
    model: String,
    #[pyo3(get)]
    os_version: String,
    #[pyo3(get)]
    hostname: String,
    #[pyo3(get)]
    uptime: String,
}

impl From<DeviceInfo> for PyDeviceInfo {
    fn from(info: DeviceInfo) -> Self {
        Self {
            vendor: info.vendor,
            model: info.model,
            os_version: info.os_version,
            hostname: info.hostname,
            uptime: info.uptime,
        }
    }
}

/// Python wrapper for NetworkDeviceConnection
#[pyclass]
struct PyNetworkDevice {
    device: Box<dyn NetworkDeviceConnection + Send>,
}

#[pymethods]
impl PyNetworkDevice {
    /// Create a new device from config
    #[staticmethod]
    fn create(config: &PyDeviceConfig) -> PyResult<Self> {
        let rust_config = py_config_to_rust_config(config);
        let device = DeviceFactory::create_device(&rust_config)
            .map_err(netssh_error_to_pyerr)?;
        
        Ok(Self {
            device,
        })
    }

    /// Connect to the device
    fn connect(&mut self) -> PyResult<()> {
        self.device.connect()
            .map_err(netssh_error_to_pyerr)
    }

    /// Close the connection to the device
    fn close(&mut self) -> PyResult<()> {
        self.device.close()
            .map_err(netssh_error_to_pyerr)
    }

    /// Check if the device is in configuration mode
    fn check_config_mode(&mut self) -> PyResult<bool> {
        self.device.check_config_mode()
            .map_err(netssh_error_to_pyerr)
    }

    /// Enter configuration mode
    fn enter_config_mode(&mut self, config_command: Option<&str>) -> PyResult<()> {
        self.device.enter_config_mode(config_command)
            .map_err(netssh_error_to_pyerr)
    }

    /// Exit configuration mode
    fn exit_config_mode(&mut self, exit_command: Option<&str>) -> PyResult<()> {
        self.device.exit_config_mode(exit_command)
            .map_err(netssh_error_to_pyerr)
    }

    /// Prepare the session after connection
    fn session_preparation(&mut self) -> PyResult<()> {
        self.device.session_preparation()
            .map_err(netssh_error_to_pyerr)
    }

    /// Configure terminal settings
    fn terminal_settings(&mut self) -> PyResult<()> {
        self.device.terminal_settings()
            .map_err(netssh_error_to_pyerr)
    }

    /// Set terminal width
    fn set_terminal_width(&mut self, width: u32) -> PyResult<()> {
        self.device.set_terminal_width(width)
            .map_err(netssh_error_to_pyerr)
    }

    /// Disable paging
    fn disable_paging(&mut self) -> PyResult<()> {
        self.device.disable_paging()
            .map_err(netssh_error_to_pyerr)
    }

    /// Set base prompt
    fn set_base_prompt(&mut self) -> PyResult<String> {
        self.device.set_base_prompt()
            .map_err(netssh_error_to_pyerr)
    }

    /// Save or commit configuration
    fn save_configuration(&mut self) -> PyResult<()> {
        self.device.save_configuration()
            .map_err(netssh_error_to_pyerr)
    }

    /// Send command to device
    fn send_command(&mut self, command: &str) -> PyResult<String> {
        self.device.send_command(command)
            .map_err(netssh_error_to_pyerr)
    }

    /// Get the device type (vendor and model)
    fn get_device_type(&self) -> &str {
        self.device.get_device_type()
    }

    /// Context manager support - enter
    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    /// Context manager support - exit
    fn __exit__(
        &mut self,
        _exc_type: Option<&PyAny>,
        _exc_value: Option<&PyAny>,
        _traceback: Option<&PyAny>,
    ) -> PyResult<bool> {
        self.close()?;
        Ok(false)  // Don't suppress exceptions
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
    output: String,
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

#[pymethods]
impl PyCommandResult {
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
    #[pyo3(get)]
    device_count: usize,
    #[pyo3(get)]
    command_count: usize,
    #[pyo3(get)]
    success_count: usize,
    #[pyo3(get)]
    failure_count: usize,
    #[pyo3(get)]
    timeout_count: usize,
    #[pyo3(get)]
    skipped_count: usize,
    #[pyo3(get)]
    start_time: String,
    #[pyo3(get)]
    end_time: String,
    #[pyo3(get)]
    duration_ms: u64,
    // Internal storage for results
    results: BatchCommandResults,
}

impl From<BatchCommandResults> for PyBatchCommandResults {
    fn from(results: BatchCommandResults) -> Self {
        Self {
            device_count: results.device_count,
            command_count: results.command_count,
            success_count: results.success_count,
            failure_count: results.failure_count,
            timeout_count: results.timeout_count,
            skipped_count: results.skipped_count,
            start_time: results.start_time.to_rfc3339(),
            end_time: results.end_time.to_rfc3339(),
            duration_ms: results.duration_ms,
            results,
        }
    }
}

#[pymethods]
impl PyBatchCommandResults {
    /// Get all results for a specific device
    fn get_device_results<'py>(&self, py: Python<'py>, device_id: &str) -> PyResult<Option<&'py PyList>> {
        match self.results.get_device_results(device_id) {
            Some(results) => {
                let py_list = PyList::empty(py);
                for result in results {
                    let py_result = PyCommandResult::from(result.clone());
                    py_list.append(Py::new(py, py_result)?)?;
                }
                Ok(Some(py_list))
            }
            None => Ok(None),
        }
    }

    /// Get all results across all devices
    fn get_all_results<'py>(&self, py: Python<'py>) -> PyResult<&'py PyList> {
        let py_list = PyList::empty(py);
        
        for device_results in self.results.results.values() {
            for result in device_results {
                let py_result = PyCommandResult::from(result.clone());
                py_list.append(Py::new(py, py_result)?)?;
            }
        }
        
        Ok(py_list)
    }
    
    /// Get all successful results
    fn get_successful_results<'py>(&self, py: Python<'py>) -> PyResult<&'py PyList> {
        let py_list = PyList::empty(py);
        
        for result in self.results.successful_results() {
            let py_result = PyCommandResult::from(result.clone());
            py_list.append(Py::new(py, py_result)?)?;
        }
        
        Ok(py_list)
    }
    
    /// Get all failed results
    fn get_failed_results<'py>(&self, py: Python<'py>) -> PyResult<&'py PyList> {
        let py_list = PyList::empty(py);
        
        for result in self.results.failed_results() {
            let py_result = PyCommandResult::from(result.clone());
            py_list.append(Py::new(py, py_result)?)?;
        }
        
        Ok(py_list)
    }
    
    /// Get results for a specific command across all devices
    fn get_command_results<'py>(&self, py: Python<'py>, command: &str) -> PyResult<&'py PyList> {
        let py_list = PyList::empty(py);
        
        for result in self.results.get_command_results(command) {
            let py_result = PyCommandResult::from(result.clone());
            py_list.append(Py::new(py, py_result)?)?;
        }
        
        Ok(py_list)
    }
    
    /// Format results as a table
    fn format_as_table(&self) -> String {
        netssh_core::parallel_execution::utils::format_as_table(&self.results)
    }
    
    /// Convert results to JSON
    fn to_json(&self) -> PyResult<String> {
        netssh_core::parallel_execution::utils::to_json(&self.results)
            .map_err(|e| PyRuntimeError::new_err(format!("JSON serialization error: {}", e)))
    }
    
    /// Convert results to CSV
    fn to_csv(&self) -> String {
        netssh_core::parallel_execution::utils::to_csv(&self.results)
    }
    
    /// Compare outputs for the same command across devices
    fn compare_outputs<'py>(&self, py: Python<'py>, command: &str) -> PyResult<&'py PyDict> {
        let output_groups = netssh_core::parallel_execution::utils::compare_outputs(&self.results, command);
        let dict = PyDict::new(py);
        
        for (output, devices) in output_groups {
            let py_devices = PyList::new(py, &devices);
            dict.set_item(output, py_devices)?;
        }
        
        Ok(dict)
    }
    
    /// Convert to a Python dictionary
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        let dict = PyDict::new(py);
        dict.set_item("device_count", self.device_count)?;
        dict.set_item("command_count", self.command_count)?;
        dict.set_item("success_count", self.success_count)?;
        dict.set_item("failure_count", self.failure_count)?;
        dict.set_item("timeout_count", self.timeout_count)?;
        dict.set_item("skipped_count", self.skipped_count)?;
        dict.set_item("start_time", &self.start_time)?;
        dict.set_item("end_time", &self.end_time)?;
        dict.set_item("duration_ms", self.duration_ms)?;
        
        let results_dict = PyDict::new(py);
        for (device_id, device_results) in &self.results.results {
            let py_results = PyList::empty(py);
            for result in device_results {
                let py_result = PyCommandResult::from(result.clone());
                py_results.append(Py::new(py, py_result)?)?;
            }
            results_dict.set_item(device_id, py_results)?;
        }
        dict.set_item("results", results_dict)?;
        
        Ok(dict)
    }
}

/// Python wrapper for ParallelExecutionManager
#[pyclass]
struct PyParallelExecutionManager {
    manager: ParallelExecutionManager,
}

#[pymethods]
impl PyParallelExecutionManager {
    /// Create a new ParallelExecutionManager
    #[new]
    fn new(
        max_concurrency: Option<usize>,
        command_timeout_seconds: Option<u64>,
        connection_timeout_seconds: Option<u64>,
        failure_strategy: Option<&str>,
        reuse_connections: Option<bool>,
    ) -> Self {
        let mut config = ParallelExecutionConfig::default();
        
        if let Some(concurrency) = max_concurrency {
            config.max_concurrency = concurrency;
        }
        
        if let Some(timeout) = command_timeout_seconds {
            config.command_timeout = Some(Duration::from_secs(timeout));
        }
        
        if let Some(timeout) = connection_timeout_seconds {
            config.connection_timeout = Some(Duration::from_secs(timeout));
        }
        
        if let Some(strategy) = failure_strategy {
            config.failure_strategy = match strategy {
                "continue_device" => FailureStrategy::ContinueDevice,
                "stop_device" => FailureStrategy::StopDevice,
                "stop_all" => FailureStrategy::StopAll,
                _ => FailureStrategy::ContinueDevice,
            };
        }
        
        if let Some(reuse) = reuse_connections {
            config.reuse_connections = reuse;
        }
        
        Self {
            manager: ParallelExecutionManager::with_config(config),
        }
    }
    
    /// Set the maximum concurrency
    fn set_max_concurrency(&mut self, max_concurrency: usize) {
        self.manager.set_max_concurrency(max_concurrency);
    }
    
    /// Set the command timeout in seconds
    fn set_command_timeout(&mut self, timeout_seconds: u64) {
        self.manager.set_command_timeout(Duration::from_secs(timeout_seconds));
    }
    
    /// Set the connection timeout in seconds
    fn set_connection_timeout(&mut self, timeout_seconds: u64) {
        self.manager.set_connection_timeout(Duration::from_secs(timeout_seconds));
    }
    
    /// Set the failure strategy
    fn set_failure_strategy(&mut self, strategy: &str) {
        let strategy = match strategy {
            "continue_device" => FailureStrategy::ContinueDevice,
            "stop_device" => FailureStrategy::StopDevice,
            "stop_all" => FailureStrategy::StopAll,
            _ => FailureStrategy::ContinueDevice,
        };
        
        self.manager.set_failure_strategy(strategy);
    }
    
    /// Set whether to reuse connections
    fn set_reuse_connections(&mut self, reuse: bool) {
        self.manager.set_reuse_connections(reuse);
    }
    
    /// Execute a command on all devices
    fn execute_command_on_all(&mut self, devices: Vec<PyDeviceConfig>, command: &str) -> PyResult<PyBatchCommandResults> {
        // Convert Python device configs to Rust device configs
        let rust_devices: Vec<DeviceConfig> = devices.iter()
            .map(|config| py_config_to_rust_config(config))
            .collect();
        
        let command = command.to_string();
        
        // Execute commands in a tokio runtime
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create Tokio runtime: {}", e)))?;
        
        let results = rt.block_on(async {
            self.manager.execute_command_on_all(rust_devices, command).await
        })
        .map_err(netssh_error_to_pyerr)?;
        
        Ok(PyBatchCommandResults::from(results))
    }
    
    /// Execute multiple commands sequentially on all devices in parallel
    fn execute_commands_on_all(&mut self, devices: Vec<PyDeviceConfig>, commands: Vec<String>) -> PyResult<PyBatchCommandResults> {
        // Convert Python device configs to Rust device configs
        let rust_devices: Vec<DeviceConfig> = devices.iter()
            .map(|config| py_config_to_rust_config(config))
            .collect();
        
        // Execute commands in a tokio runtime
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create Tokio runtime: {}", e)))?;
        
        let results = rt.block_on(async {
            self.manager.execute_commands_on_all(rust_devices, commands).await
        })
        .map_err(netssh_error_to_pyerr)?;
        
        Ok(PyBatchCommandResults::from(results))
    }
    
    /// Execute different commands on different devices
    fn execute_commands<'py>(&mut self, py: Python<'py>, device_commands: &PyDict) -> PyResult<PyBatchCommandResults> {
        let mut rust_device_commands: HashMap<DeviceConfig, Vec<String>> = HashMap::new();
        
        for (key, value) in device_commands.iter() {
            let py_device_config = key.extract::<PyDeviceConfig>()?;
            let rust_device_config = py_config_to_rust_config(&py_device_config);
            
            let commands = value.extract::<Vec<String>>()?;
            rust_device_commands.insert(rust_device_config, commands);
        }
        
        // Execute commands in a tokio runtime
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create Tokio runtime: {}", e)))?;
        
        let results = rt.block_on(async {
            self.manager.execute_commands(rust_device_commands).await
        })
        .map_err(netssh_error_to_pyerr)?;
        
        Ok(PyBatchCommandResults::from(results))
    }
    
    /// Clean up resources
    fn cleanup(&mut self) {
        self.manager.cleanup();
    }
    
    /// Context manager support - enter
    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    
    /// Context manager support - exit
    fn __exit__(
        &mut self,
        _exc_type: Option<&PyAny>,
        _exc_value: Option<&PyAny>,
        _traceback: Option<&PyAny>,
    ) -> PyResult<bool> {
        self.cleanup();
        Ok(false)  // Don't suppress exceptions
    }
} 