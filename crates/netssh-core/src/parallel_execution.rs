use crate::command_result::{BatchCommandResults, CommandResult, CommandStatus};
use crate::device_connection::{DeviceConfig, NetworkDeviceConnection};
use crate::device_factory::DeviceFactory;
use crate::error::NetsshError;
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;
use tracing::{debug, error, instrument, warn};

/// Batch execution strategy for handling failures
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FailureStrategy {
    /// Continue executing commands on the device, skipping only the failed command
    ContinueOnDevice,
    /// Skip remaining commands for the device where the failure occurred
    SkipDevice,
    /// Abort the entire batch operation across all devices
    AbortBatch,
}

/// Configuration for parallel execution
#[derive(Debug, Clone)]
pub struct ParallelExecutionConfig {
    /// Maximum number of concurrent connections (default: 10)
    pub max_concurrent_devices: usize,
    /// Strategy to handle command failures (default: SkipDevice)
    pub failure_strategy: FailureStrategy,
    /// Command timeout in seconds (default: 30)
    pub command_timeout: Duration,
    /// Whether to stop on first failure (default: false)
    pub stop_on_first_failure: bool,
}

impl Default for ParallelExecutionConfig {
    fn default() -> Self {
        Self {
            max_concurrent_devices: 10,
            failure_strategy: FailureStrategy::SkipDevice,
            command_timeout: Duration::from_secs(30),
            stop_on_first_failure: false,
        }
    }
}

/// Manager for parallel execution of commands on multiple devices
pub struct ParallelExecutionManager {
    /// Configuration for parallel execution
    config: ParallelExecutionConfig,
    /// Semaphore for limiting concurrency
    concurrency_semaphore: Arc<Semaphore>,
    /// Active connections that can be reused
    active_connections: HashMap<String, Box<dyn NetworkDeviceConnection + Send>>,
}

impl ParallelExecutionManager {
    /// Create a new ParallelExecutionManager with the default configuration
    #[instrument(level = "debug")]
    pub fn new() -> Self {
        debug!("Creating new ParallelExecutionManager with default config");
        Self::with_config(ParallelExecutionConfig::default())
    }

    /// Create a new ParallelExecutionManager with a custom configuration
    #[instrument(skip(config), level = "debug")]
    pub fn with_config(config: ParallelExecutionConfig) -> Self {
        debug!("Creating new ParallelExecutionManager with custom config");
        Self {
            concurrency_semaphore: Arc::new(Semaphore::new(config.max_concurrent_devices)),
            config,
            active_connections: HashMap::new(),
        }
    }

    /// Set the maximum concurrency
    pub fn set_max_concurrency(&mut self, max_concurrency: usize) {
        self.config.max_concurrent_devices = max_concurrency;
        self.concurrency_semaphore = Arc::new(Semaphore::new(max_concurrency));
    }

    /// Set the command timeout
    pub fn set_command_timeout(&mut self, timeout: Duration) {
        self.config.command_timeout = timeout;
    }

    /// Set the failure strategy
    pub fn set_failure_strategy(&mut self, strategy: FailureStrategy) {
        self.config.failure_strategy = strategy;
    }

    /// Set whether to reuse connections
    pub fn set_reuse_connections(&mut self, reuse: bool) {
        self.config.stop_on_first_failure = !reuse;
    }

    /// Execute a command on all devices
    pub async fn execute_command_on_all(
        &mut self,
        devices: Vec<DeviceConfig>,
        command: String,
    ) -> Result<BatchCommandResults, NetsshError> {
        // Create a map of device IDs to commands
        let device_commands: HashMap<DeviceConfig, Vec<String>> = devices
            .into_iter()
            .map(|device| (device, vec![command.clone()]))
            .collect();

        // Execute the commands
        self.execute_commands(device_commands).await
    }

    /// Execute multiple commands sequentially on all devices in parallel
    pub async fn execute_commands_on_all(
        &mut self,
        devices: Vec<DeviceConfig>,
        commands: Vec<String>,
    ) -> Result<BatchCommandResults, NetsshError> {
        // Create a map of device IDs to commands
        let device_commands: HashMap<DeviceConfig, Vec<String>> = devices
            .into_iter()
            .map(|device| (device, commands.clone()))
            .collect();

        // Execute the commands
        self.execute_commands(device_commands).await
    }

    /// Execute different commands on different devices
    pub async fn execute_commands(
        &mut self,
        device_commands: HashMap<DeviceConfig, Vec<String>>,
    ) -> Result<BatchCommandResults, NetsshError> {
        // Initialize batch results
        let mut batch_results = BatchCommandResults::new();
        batch_results.start_time = Utc::now();

        // Keep track of failed devices if using SkipDevice strategy
        let mut failed_devices = HashSet::new();

        // Set of all tasks
        let mut tasks = Vec::new();

        // Start a task for each device
        for (device_config, commands) in device_commands {
            // Skip devices that have failed in SkipDevice strategy
            if self.config.failure_strategy == FailureStrategy::SkipDevice
                && failed_devices.contains(&device_config.host)
            {
                continue;
            }

            // Clone values needed for the async block
            let semaphore = Arc::clone(&self.concurrency_semaphore);
            let host = device_config.host.clone();
            let device_type = device_config.device_type.clone();
            let command_timeout = self.config.command_timeout;
            let failure_strategy = self.config.failure_strategy;
            let reuse_connection = self.config.stop_on_first_failure;

            // Get a reused connection or None
            let mut connection = if reuse_connection && self.active_connections.contains_key(&host)
            {
                debug!("Reusing existing connection for device {}", host);
                Some(self.active_connections.remove(&host).unwrap())
            } else {
                None
            };

            // Save the host value to use when adding to tasks
            let task_host = host.clone();

            // Spawn a task for this device
            let handle: JoinHandle<Vec<CommandResult>> = tokio::spawn(async move {
                // Acquire a permit from the semaphore
                let _permit = semaphore.acquire().await.unwrap();
                let mut results = Vec::new();

                // Initialize the connection if not reusing one
                if connection.is_none() {
                    debug!("Creating new connection for device {}", host);
                    match DeviceFactory::create_device(&device_config) {
                        Ok(new_connection) => {
                            connection = Some(new_connection);
                        }
                        Err(err) => {
                            error!("Failed to create connection for device {}: {}", host, err);
                            // For each command, create a failure result
                            for cmd in commands {
                                results.push(CommandResult::failure(
                                    host.clone(),
                                    device_type.clone(),
                                    cmd,
                                    String::new(),
                                    Utc::now(),
                                    Utc::now(),
                                    format!("Failed to create connection: {}", err),
                                ));
                            }
                            return results;
                        }
                    }
                }

                let mut connection = connection.unwrap();

                // Connect to the device if not already connected
                match connection.connect() {
                    Ok(_) => {
                        // Execute each command sequentially
                        let mut device_failed = false;

                        for cmd in commands {
                            // Skip if the device has already failed and we're using SkipDevice strategy
                            if device_failed && failure_strategy == FailureStrategy::SkipDevice {
                                results.push(CommandResult::skipped(
                                    host.clone(),
                                    device_type.clone(),
                                    cmd,
                                ));
                                continue;
                            }

                            // Record start time
                            let start_time = Utc::now();

                            // Absolute timeout in seconds, or use device_config timeout if not specified
                            let timeout_duration = if let Some(timeout) = device_config.timeout {
                                timeout
                            } else {
                                command_timeout
                            };

                            // Execute the command with timeout
                            let result = tokio::time::timeout(timeout_duration, async {
                                connection.send_command(&cmd)
                            })
                            .await;

                            let end_time = Utc::now();

                            match result {
                                Ok(Ok(output)) => {
                                    // Command succeeded
                                    results.push(CommandResult::success(
                                        host.clone(),
                                        device_type.clone(),
                                        cmd,
                                        output,
                                        start_time,
                                        end_time,
                                    ));
                                }
                                Ok(Err(err)) => {
                                    // Command failed due to error
                                    device_failed = true;
                                    results.push(CommandResult::failure(
                                        host.clone(),
                                        device_type.clone(),
                                        cmd,
                                        String::new(),
                                        start_time,
                                        end_time,
                                        format!("Command execution error: {}", err),
                                    ));
                                }
                                Err(_) => {
                                    // Command timed out
                                    device_failed = true;
                                    results.push(CommandResult::timeout(
                                        host.clone(),
                                        device_type.clone(),
                                        cmd,
                                        start_time,
                                    ));
                                }
                            }
                        }
                    }
                    Err(err) => {
                        error!("Failed to connect to device {}: {}", host, err);
                        // For each command, create a failure result
                        for cmd in commands {
                            results.push(CommandResult::failure(
                                host.clone(),
                                device_type.clone(),
                                cmd,
                                String::new(),
                                Utc::now(),
                                Utc::now(),
                                format!("Failed to connect: {}", err),
                            ));
                        }
                    }
                }

                results
            });

            tasks.push((task_host, handle));
        }

        // Process the results of each task
        let mut stop_all = false;

        for (host, handle) in tasks {
            // Skip if we're stopping all execution
            if stop_all {
                continue;
            }

            // Wait for the task to complete
            match handle.await {
                Ok(results) => {
                    // Check if any commands failed
                    let has_failures = results.iter().any(|r| r.status == CommandStatus::Failed);

                    // Add results to batch results
                    for result in results {
                        let should_stop = result.status == CommandStatus::Failed
                            && self.config.failure_strategy == FailureStrategy::AbortBatch;

                        batch_results.add_result(result);

                        // Check if we need to stop all execution
                        if should_stop {
                            stop_all = true;
                            break;
                        }
                    }

                    // Track failed devices for SkipDevice strategy
                    if has_failures && self.config.failure_strategy == FailureStrategy::SkipDevice {
                        failed_devices.insert(host);
                    }
                }
                Err(err) => {
                    error!("Task for device {} panicked: {}", host, err);
                }
            }
        }

        // Complete the batch results
        batch_results.complete();

        Ok(batch_results)
    }

    /// Clean up any active connections
    pub fn cleanup(&mut self) {
        for (host, mut connection) in self.active_connections.drain() {
            if let Err(err) = connection.close() {
                warn!("Error closing connection to {}: {}", host, err);
            }
        }
    }
}

impl Drop for ParallelExecutionManager {
    fn drop(&mut self) {
        self.cleanup();
    }
}
