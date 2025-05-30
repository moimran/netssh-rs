use chrono::Utc;
use netssh_core::command_result::{BatchCommandResults, CommandResult, ParseOptions};
use netssh_core::device_connection::{DeviceConfig, NetworkDeviceConnection};
use netssh_core::device_factory::DeviceFactory;
use netssh_core::error::NetsshError;
use netssh_core::{FailureStrategy, ParallelExecutionConfig, ParallelExecutionManager};
use shared_config::WorkspaceConfig;

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

/// Python wrapper for ParseOptions
#[pyclass]
#[derive(Clone)]
struct PyParseOptions {
    #[pyo3(get, set)]
    enabled: bool,
    #[pyo3(get, set)]
    template_dir: Option<String>,
}

#[pymethods]
impl PyParseOptions {
    #[new]
    #[pyo3(signature = (enabled=false, template_dir=None))]
    fn new(enabled: bool, template_dir: Option<String>) -> Self {
        Self {
            enabled,
            template_dir,
        }
    }

    /// Create ParseOptions with parsing enabled
    #[staticmethod]
    fn enabled() -> Self {
        Self {
            enabled: true,
            template_dir: None,
        }
    }

    /// Create ParseOptions with custom template directory
    #[staticmethod]
    fn with_template_dir(template_dir: String) -> Self {
        Self {
            enabled: true,
            template_dir: Some(template_dir),
        }
    }
}

impl From<PyParseOptions> for ParseOptions {
    fn from(py_options: PyParseOptions) -> Self {
        Self {
            enabled: py_options.enabled,
            template_dir: py_options.template_dir,
        }
    }
}

/// Python wrapper for WorkspaceConfig
///
/// This class provides access to the unified workspace configuration
/// that contains settings for all netssh-rs crates.
#[pyclass]
struct PyWorkspaceConfig {
    config: WorkspaceConfig,
}

#[pymethods]
impl PyWorkspaceConfig {
    /// Load configuration from environment variables and config files
    #[staticmethod]
    fn load() -> PyResult<Self> {
        let config = WorkspaceConfig::load()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to load config: {}", e)))?;
        Ok(Self { config })
    }

    /// Create a default configuration
    #[staticmethod]
    fn default() -> PyResult<Self> {
        // Try to load config, fall back to creating a minimal default if that fails
        let config = WorkspaceConfig::load().unwrap_or_else(|_| {
            // Create a minimal default configuration
            shared_config::WorkspaceConfig {
                global: shared_config::GlobalConfig {
                    log_level: "info".to_string(),
                    environment: "development".to_string(),
                    default_timeout_seconds: 30,
                },
                scheduler: shared_config::SchedulerConfig {
                    enabled: false,
                    database: shared_config::DatabaseConfig {
                        url: "sqlite::memory:".to_string(),
                        max_connections: 1,
                    },
                    server: shared_config::ServerConfig {
                        host: "127.0.0.1".to_string(),
                        port: 8080,
                    },
                    worker: shared_config::WorkerConfig {
                        concurrency: 1,
                        timeout_seconds: 300,
                        connection_reuse: false,
                        max_idle_time_seconds: 300,
                        max_connections_per_worker: 1,
                        failure_strategy: shared_config::FailureStrategy::ContinueOnFailure,
                        failure_strategy_n: 3,
                    },
                    board: shared_config::BoardConfig {
                        enabled: false,
                        ui_path: "/board".to_string(),
                        api_prefix: "/board/api".to_string(),
                        auth_enabled: false,
                    },
                    logging: shared_config::LoggingConfig {
                        level: "info".to_string(),
                        file: None,
                        format: None,
                        rotation: None,
                    },
                    scheduler: shared_config::SchedulerServiceConfig {
                        enabled: false,
                        poll_interval_seconds: 30,
                        timezone: None,
                        max_concurrent_jobs: 1,
                    },
                },
                netssh: shared_config::NetsshConfig {
                    network: shared_config::NetworkSettings {
                        tcp_connect_timeout_secs: 60,
                        tcp_read_timeout_secs: 30,
                        tcp_write_timeout_secs: 30,
                        default_ssh_port: 22,
                        command_response_timeout_secs: 30,
                        pattern_match_timeout_secs: 20,
                        command_exec_delay_ms: 100,
                        retry_delay_ms: 1000,
                        max_retry_attempts: 3,
                        device_operation_timeout_secs: 120,
                    },
                    ssh: shared_config::SshSettings {
                        blocking_timeout_secs: 1,
                        auth_timeout_secs: 30,
                        keepalive_interval_secs: 60,
                        channel_open_timeout_secs: 20,
                    },
                    buffer: shared_config::BufferSettings {
                        read_buffer_size: 16384,
                        buffer_pool_size: 32,
                        buffer_reuse_threshold: 16384,
                        auto_clear_buffer: true,
                    },
                    concurrency: shared_config::ConcurrencySettings {
                        max_connections: 1,
                        permit_acquire_timeout_ms: 5000,
                        connection_idle_timeout_secs: 300,
                    },
                    logging: shared_config::NetsshLoggingConfig {
                        enable_session_log: false,
                        session_log_path: "logs".to_string(),
                        log_binary_data: false,
                    },
                    security: shared_config::SecurityConfig {
                        strict_host_key_checking: false,
                        known_hosts_file: "~/.ssh/known_hosts".to_string(),
                        max_auth_attempts: 3,
                    },
                },
                textfsm: shared_config::TextFsmConfig {
                    template_cache_size: 1000,
                    parsing_timeout_seconds: 10,
                    template_directories: vec!["templates/".to_string()],
                    enable_caching: true,
                },
            }
        });

        Ok(Self { config })
    }

    /// Get the global configuration section
    #[getter]
    fn global_config(&self) -> PyResult<String> {
        serde_json::to_string_pretty(&self.config.global)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to serialize global config: {}", e)))
    }

    /// Get the netssh configuration section
    #[getter]
    fn netssh_config(&self) -> PyResult<String> {
        serde_json::to_string_pretty(&self.config.netssh)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to serialize netssh config: {}", e)))
    }

    /// Get the textfsm configuration section
    #[getter]
    fn textfsm_config(&self) -> PyResult<String> {
        serde_json::to_string_pretty(&self.config.textfsm)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to serialize textfsm config: {}", e)))
    }

    /// Get the scheduler configuration section
    #[getter]
    fn scheduler_config(&self) -> PyResult<String> {
        serde_json::to_string_pretty(&self.config.scheduler)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to serialize scheduler config: {}", e)))
    }

    /// Get the entire configuration as JSON
    fn to_json(&self) -> PyResult<String> {
        serde_json::to_string_pretty(&self.config)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to serialize config: {}", e)))
    }

    /// Get a specific configuration value by path (e.g., "netssh.default_ssh_timeout")
    fn get_value(&self, path: &str) -> PyResult<String> {
        // Simple path-based access to configuration values
        let parts: Vec<&str> = path.split('.').collect();

        match parts.as_slice() {
            ["global", "log_level"] => Ok(self.config.global.log_level.clone()),
            ["global", "environment"] => Ok(self.config.global.environment.clone()),
            ["netssh", "network", "tcp_connect_timeout_secs"] => Ok(self.config.netssh.network.tcp_connect_timeout_secs.to_string()),
            ["netssh", "network", "command_response_timeout_secs"] => Ok(self.config.netssh.network.command_response_timeout_secs.to_string()),
            ["netssh", "network", "default_ssh_port"] => Ok(self.config.netssh.network.default_ssh_port.to_string()),
            ["netssh", "buffer", "read_buffer_size"] => Ok(self.config.netssh.buffer.read_buffer_size.to_string()),
            ["netssh", "network", "max_retry_attempts"] => Ok(self.config.netssh.network.max_retry_attempts.to_string()),
            ["netssh", "concurrency", "max_connections"] => Ok(self.config.netssh.concurrency.max_connections.to_string()),
            ["textfsm", "template_cache_size"] => Ok(self.config.textfsm.template_cache_size.to_string()),
            ["textfsm", "parsing_timeout_seconds"] => Ok(self.config.textfsm.parsing_timeout_seconds.to_string()),
            ["textfsm", "enable_caching"] => Ok(self.config.textfsm.enable_caching.to_string()),
            _ => Err(PyValueError::new_err(format!("Unknown configuration path: {}", path))),
        }
    }

    /// Get template directories from TextFSM configuration
    #[getter]
    fn template_directories(&self) -> Vec<String> {
        self.config.textfsm.template_directories.clone()
    }

    /// String representation of the configuration
    fn __str__(&self) -> String {
        format!("WorkspaceConfig(environment={}, log_level={})",
                self.config.global.environment,
                self.config.global.log_level)
    }
}

/// Python module for netssh-rs
#[pymodule]
fn netssh_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Register custom exception
    m.add_class::<PyConnectionError>()?;

    // Add classes
    m.add_class::<PyDeviceConfig>()?;
    m.add_class::<PyNetworkDevice>()?;
    m.add_class::<PyCommandResult>()?;
    m.add_class::<PyBatchCommandResults>()?;
    m.add_class::<PyParallelExecutionManager>()?;
    m.add_class::<PyParseOptions>()?;
    m.add_class::<PyWorkspaceConfig>()?;

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
#[pyclass(unsendable)]
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
        let mut cmd_builder = self.device.send_command(command);

        if let Some(timeout) = read_timeout {
            cmd_builder = cmd_builder.timeout(timeout);
        }
        if let Some(strip_p) = strip_prompt {
            cmd_builder = cmd_builder.strip_prompt(strip_p);
        }
        if let Some(strip_c) = strip_command {
            cmd_builder = cmd_builder.strip_command(strip_c);
        }
        if let Some(norm) = normalize {
            cmd_builder = cmd_builder.normalize(norm);
        }
        if let Some(verify) = cmd_verify {
            cmd_builder = cmd_builder.cmd_verify(verify);
        }
        if let Some(expect) = expect_string {
            cmd_builder = cmd_builder.expect_string(expect);
        }
        if let Some(auto_find) = auto_find_prompt {
            cmd_builder = cmd_builder.auto_find_prompt(auto_find);
        }

        let result = cmd_builder.execute();
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
        _exc_type: Option<&Bound<'_, PyAny>>,
        _exc_value: Option<&Bound<'_, PyAny>>,
        _traceback: Option<&Bound<'_, PyAny>>,
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
        config_commands: &Bound<'_, PyList>,
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
        let mut config_builder = self.device.send_config_set(commands);

        if let Some(exit_mode) = exit_config_mode {
            config_builder = config_builder.exit_config_mode(exit_mode);
        }
        if let Some(timeout) = read_timeout {
            config_builder = config_builder.timeout(timeout);
        }
        if let Some(strip_p) = strip_prompt {
            config_builder = config_builder.strip_prompt(strip_p);
        }
        if let Some(strip_c) = strip_command {
            config_builder = config_builder.strip_command(strip_c);
        }
        if let Some(config_cmd) = config_mode_command {
            config_builder = config_builder.config_mode_command(config_cmd);
        }
        if let Some(verify) = cmd_verify {
            config_builder = config_builder.cmd_verify(verify);
        }
        if let Some(enter_mode) = enter_config_mode {
            config_builder = config_builder.enter_config_mode(enter_mode);
        }
        if let Some(error_pat) = error_pattern {
            config_builder = config_builder.error_pattern(error_pat);
        }
        if let Some(term) = terminator {
            config_builder = config_builder.terminator(term);
        }
        if let Some(bypass) = bypass_commands {
            config_builder = config_builder.bypass_commands(bypass);
        }
        if let Some(fast) = fast_cli {
            config_builder = config_builder.fast_cli(fast);
        }

        let result = config_builder.execute();
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
/// - parse_status: The status of TextFSM parsing (NotAttempted, Success, Failed, NoTemplate)
/// - parsed_data: Structured data from TextFSM parsing (if successful)
/// - parse_error: Error message if parsing failed
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
    #[pyo3(get)]
    parse_status: String,
    #[pyo3(get)]
    parsed_data: Option<String>,
    #[pyo3(get)]
    parse_error: Option<String>,
}

impl From<CommandResult> for PyCommandResult {
    fn from(result: CommandResult) -> Self {
        // Convert parsed data to JSON string if available
        let parsed_data_json = result.parsed_data_as_json()
            .and_then(|json_result| json_result.ok());

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
            parse_status: format!("{:?}", result.parse_status),
            parsed_data: parsed_data_json,
            parse_error: result.parse_error,
        }
    }
}



#[pymethods]
impl PyCommandResult {
    /// Convert the command result to a Python dictionary
    ///
    /// Returns:
    ///     dict: Dictionary representation of the command result
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
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
        dict.set_item("parse_status", &self.parse_status)?;
        dict.set_item("parsed_data", &self.parsed_data)?;
        dict.set_item("parse_error", &self.parse_error)?;
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

    /// Check if TextFSM parsing was successful
    ///
    /// Returns:
    ///     bool: True if parsing was successful, False otherwise
    fn is_parsed(&self) -> bool {
        self.parse_status == "Success"
    }

    /// Check if TextFSM parsing was attempted
    ///
    /// Returns:
    ///     bool: True if parsing was attempted, False otherwise
    fn parse_attempted(&self) -> bool {
        self.parse_status != "NotAttempted"
    }

    /// Get parsed data as JSON string
    ///
    /// Returns:
    ///     str: Parsed data as JSON string, or None if not available
    fn get_parsed_data_json(&self) -> Option<&str> {
        self.parsed_data.as_deref()
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
    ) -> PyResult<Option<Bound<'py, PyList>>> {
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
    fn get_all_results<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
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
    fn get_successful_results<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
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
    fn get_failed_results<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
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
    fn get_command_results<'py>(&self, py: Python<'py>, command: &str) -> PyResult<Bound<'py, PyList>> {
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
    fn compare_outputs<'py>(&self, py: Python<'py>, command: &str) -> PyResult<Bound<'py, PyDict>> {
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
#[pyclass(unsendable)]
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
        configs: &Bound<'_, PyList>,
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
        configs: &Bound<'_, PyList>,
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
        device_commands: &Bound<'_, PyDict>,
    ) -> PyResult<PyBatchCommandResults> {
        // Extract device commands map outside the allow_threads block
        let mut device_map = HashMap::new();

        for (k, v) in device_commands.iter() {
            // Get the PyDeviceConfig
            let config = match k.extract::<PyDeviceConfig>() {
                Ok(py_config) => py_config_to_rust_config(&py_config),
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
        _exc_type: Option<&Bound<'_, PyAny>>,
        _exc_value: Option<&Bound<'_, PyAny>>,
        _traceback: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<bool> {
        self.cleanup();
        Ok(false) // Don't suppress exceptions
    }
}

// Helper function to extract DeviceConfig objects from a PyList
fn extract_device_configs(configs: &Bound<'_, PyList>) -> PyResult<Vec<DeviceConfig>> {
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
