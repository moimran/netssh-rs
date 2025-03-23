use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use pyo3::exceptions::PyRuntimeError;
use std::time::Duration;

use netssh_core::device_connection::{NetworkDeviceConnection, DeviceConfig, DeviceInfo};
use netssh_core::device_factory::DeviceFactory;
use netssh_core::error::NetsshError;

/// Python module for netssh-rs
#[pymodule]
fn netssh_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyDeviceConfig>()?;
    m.add_class::<PyDeviceInfo>()?;
    m.add_class::<PyNetworkDevice>()?;
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