use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
// Note: Using chrono with serde features to enable DateTime serialization/deserialization
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;
use log::{debug, error, warn};

use crate::device_connection::{DeviceConfig, NetworkDeviceConnection};
use crate::device_factory::DeviceFactory;
use crate::error::NetsshError;
use crate::settings;

/// Represents the execution status of a command
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandStatus {
    /// Command executed successfully
    Success,
    /// Command execution failed
    Failed,
    /// Command execution timed out
    Timeout,
    /// Command execution was skipped (e.g., due to previous command failure)
    Skipped,
}

/// Holds the result of executing a command on a device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    /// Identifier for the device (hostname or IP)
    pub device_id: String,
    /// Device type (vendor/model)
    pub device_type: String,
    /// The command that was executed
    pub command: String,
    /// Output text from the command
    pub output: String,
    /// Time when command execution started
    pub start_time: DateTime<Utc>,
    /// Time when command execution ended
    pub end_time: DateTime<Utc>,
    /// Duration of command execution in milliseconds
    pub duration_ms: u64,
    /// Status of command execution
    pub status: CommandStatus,
    /// Error message if command failed
    pub error: Option<String>,
}

impl CommandResult {
    /// Create a new CommandResult for a successful command execution
    pub fn success(
        device_id: String,
        device_type: String,
        command: String,
        output: String,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Self {
        let duration = end_time.signed_duration_since(start_time);
        let duration_ms = duration.num_milliseconds() as u64;
        
        Self {
            device_id,
            device_type,
            command,
            output,
            start_time,
            end_time,
            duration_ms,
            status: CommandStatus::Success,
            error: None,
        }
    }
    
    /// Create a new CommandResult for a failed command execution
    pub fn failure(
        device_id: String,
        device_type: String,
        command: String,
        output: String,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        error: String,
    ) -> Self {
        let duration = end_time.signed_duration_since(start_time);
        let duration_ms = duration.num_milliseconds() as u64;
        
        Self {
            device_id,
            device_type,
            command,
            output,
            start_time,
            end_time,
            duration_ms,
            status: CommandStatus::Failed,
            error: Some(error),
        }
    }
    
    /// Create a new CommandResult for a timed out command execution
    pub fn timeout(
        device_id: String,
        device_type: String,
        command: String,
        start_time: DateTime<Utc>,
    ) -> Self {
        let end_time = Utc::now();
        let duration = end_time.signed_duration_since(start_time);
        let duration_ms = duration.num_milliseconds() as u64;
        
        Self {
            device_id,
            device_type,
            command,
            output: String::new(),
            start_time,
            end_time,
            duration_ms,
            status: CommandStatus::Timeout,
            error: Some(format!("Command execution timed out after {} ms", duration_ms)),
        }
    }
    
    /// Create a new CommandResult for a skipped command execution
    pub fn skipped(
        device_id: String,
        device_type: String,
        command: String,
    ) -> Self {
        let now = Utc::now();
        
        Self {
            device_id,
            device_type,
            command,
            output: String::new(),
            start_time: now,
            end_time: now,
            duration_ms: 0,
            status: CommandStatus::Skipped,
            error: Some("Command execution skipped".to_string()),
        }
    }
}

/// Batch execution strategy for handling failures
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FailureStrategy {
    /// Continue execution of remaining commands for a device even if a command fails
    ContinueDevice,
    /// Skip remaining commands for a device if any command fails for that device
    StopDevice,
    /// Stop all execution across all devices if any command fails on any device
    StopAll,
}

/// Configuration for parallel execution
#[derive(Debug, Clone)]
pub struct ParallelExecutionConfig {
    /// Maximum number of concurrent device connections
    pub max_concurrency: usize,
    /// Command execution timeout in seconds (overrides device timeout)
    pub command_timeout: Option<Duration>,
    /// Connection timeout in seconds (overrides device timeout)
    pub connection_timeout: Option<Duration>,
    /// Strategy for handling command failures
    pub failure_strategy: FailureStrategy,
    /// Whether to reuse connections for sequential commands to the same device
    pub reuse_connections: bool,
}

impl Default for ParallelExecutionConfig {
    fn default() -> Self {
        Self {
            max_concurrency: 10, // Default to 10 concurrent connections
            command_timeout: None,
            connection_timeout: None,
            failure_strategy: FailureStrategy::ContinueDevice,
            reuse_connections: true,
        }
    }
}

/// Container for all results from a batch command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCommandResults {
    /// Map of device ID to a list of command results for that device
    pub results: HashMap<String, Vec<CommandResult>>,
    /// Total number of devices processed
    pub device_count: usize,
    /// Total number of commands executed
    pub command_count: usize,
    /// Number of successful commands
    pub success_count: usize,
    /// Number of failed commands
    pub failure_count: usize,
    /// Number of timed out commands
    pub timeout_count: usize,
    /// Number of skipped commands
    pub skipped_count: usize,
    /// Time when batch execution started
    pub start_time: DateTime<Utc>,
    /// Time when batch execution ended
    pub end_time: DateTime<Utc>,
    /// Duration of batch execution in milliseconds
    pub duration_ms: u64,
}

impl BatchCommandResults {
    /// Create a new empty BatchCommandResults
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            results: HashMap::new(),
            device_count: 0,
            command_count: 0,
            success_count: 0,
            failure_count: 0,
            timeout_count: 0,
            skipped_count: 0,
            start_time: now,
            end_time: now,
            duration_ms: 0,
        }
    }
    
    /// Add a command result to the batch results
    pub fn add_result(&mut self, result: CommandResult) {
        let device_id = result.device_id.clone();
        
        // Update counters based on result status
        match result.status {
            CommandStatus::Success => self.success_count += 1,
            CommandStatus::Failed => self.failure_count += 1,
            CommandStatus::Timeout => self.timeout_count += 1,
            CommandStatus::Skipped => self.skipped_count += 1,
        }
        
        self.command_count += 1;
        
        // Add result to device's results
        self.results
            .entry(device_id)
            .or_insert_with(Vec::new)
            .push(result);
            
        // Update device count
        self.device_count = self.results.len();
    }
    
    /// Complete the batch results with timing information
    pub fn complete(&mut self) {
        self.end_time = Utc::now();
        let duration = self.end_time.signed_duration_since(self.start_time);
        self.duration_ms = duration.num_milliseconds() as u64;
    }
    
    /// Get all results for a specific device
    pub fn get_device_results(&self, device_id: &str) -> Option<&Vec<CommandResult>> {
        self.results.get(device_id)
    }
    
    /// Get all results for a specific command across all devices
    pub fn get_command_results(&self, command: &str) -> Vec<&CommandResult> {
        self.results
            .values()
            .flat_map(|results| results.iter())
            .filter(|result| result.command == command)
            .collect()
    }
    
    /// Filter results by status
    pub fn filter_by_status(&self, status: CommandStatus) -> Vec<&CommandResult> {
        self.results
            .values()
            .flat_map(|results| results.iter())
            .filter(|result| result.status == status)
            .collect()
    }
    
    /// Get successful results
    pub fn successful_results(&self) -> Vec<&CommandResult> {
        self.filter_by_status(CommandStatus::Success)
    }
    
    /// Get failed results
    pub fn failed_results(&self) -> Vec<&CommandResult> {
        self.filter_by_status(CommandStatus::Failed)
    }
    
    /// Get timed out results
    pub fn timeout_results(&self) -> Vec<&CommandResult> {
        self.filter_by_status(CommandStatus::Timeout)
    }
    
    /// Get skipped results
    pub fn skipped_results(&self) -> Vec<&CommandResult> {
        self.filter_by_status(CommandStatus::Skipped)
    }
}

/// Manager for executing commands in parallel across multiple devices
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
    pub fn new() -> Self {
        Self::with_config(ParallelExecutionConfig::default())
    }
    
    /// Create a new ParallelExecutionManager with a custom configuration
    pub fn with_config(config: ParallelExecutionConfig) -> Self {
        Self {
            concurrency_semaphore: Arc::new(Semaphore::new(config.max_concurrency)),
            config,
            active_connections: HashMap::new(),
        }
    }
    
    /// Set the maximum concurrency
    pub fn set_max_concurrency(&mut self, max_concurrency: usize) {
        self.config.max_concurrency = max_concurrency;
        self.concurrency_semaphore = Arc::new(Semaphore::new(max_concurrency));
    }
    
    /// Set the command timeout
    pub fn set_command_timeout(&mut self, timeout: Duration) {
        self.config.command_timeout = Some(timeout);
    }
    
    /// Set the connection timeout
    pub fn set_connection_timeout(&mut self, timeout: Duration) {
        self.config.connection_timeout = Some(timeout);
    }
    
    /// Set the failure strategy
    pub fn set_failure_strategy(&mut self, strategy: FailureStrategy) {
        self.config.failure_strategy = strategy;
    }
    
    /// Set whether to reuse connections
    pub fn set_reuse_connections(&mut self, reuse: bool) {
        self.config.reuse_connections = reuse;
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
        
        // Keep track of failed devices if using StopDevice strategy
        let mut failed_devices = HashSet::new();
        
        // Set of all tasks
        let mut tasks = Vec::new();
        
        // Start a task for each device
        for (device_config, commands) in device_commands {
            // Skip devices that have failed in StopDevice strategy
            if self.config.failure_strategy == FailureStrategy::StopDevice && 
               failed_devices.contains(&device_config.host) {
                continue;
            }
            
            // Clone values needed for the async block
            let semaphore = Arc::clone(&self.concurrency_semaphore);
            let host = device_config.host.clone();
            let device_type = device_config.device_type.clone();
            let connection_timeout = self.config.connection_timeout;
            let command_timeout = self.config.command_timeout;
            let failure_strategy = self.config.failure_strategy;
            let reuse_connection = self.config.reuse_connections;
            
            // Get a reused connection or None
            let mut connection = if reuse_connection && self.active_connections.contains_key(&host) {
                debug!("Reusing existing connection for device {}", host);
                Some(self.active_connections.remove(&host).unwrap())
            } else {
                None
            };
            
            // Apply connection timeout override if specified
            let mut device_config = device_config.clone();
            if let Some(timeout) = connection_timeout {
                device_config.timeout = Some(timeout);
            }
            
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
                            // Skip if the device has already failed and we're using StopDevice strategy
                            if device_failed && failure_strategy == FailureStrategy::StopDevice {
                                results.push(CommandResult::skipped(
                                    host.clone(),
                                    device_type.clone(),
                                    cmd,
                                ));
                                continue;
                            }
                            
                            // Record start time
                            let start_time = Utc::now();
                            
                            // Set command timeout
                            let timeout_duration = command_timeout.unwrap_or(
                                device_config.timeout.unwrap_or(Duration::from_secs(60))
                            );
                            
                            // Execute the command with timeout
                            let result = tokio::time::timeout(
                                timeout_duration,
                                async {
                                    connection.send_command(&cmd)
                                }
                            ).await;
                            
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
                        let should_stop = result.status == CommandStatus::Failed &&
                                          self.config.failure_strategy == FailureStrategy::StopAll;
                        
                        batch_results.add_result(result);
                        
                        // Check if we need to stop all execution
                        if should_stop {
                            stop_all = true;
                            break;
                        }
                    }
                    
                    // Track failed devices for StopDevice strategy
                    if has_failures && self.config.failure_strategy == FailureStrategy::StopDevice {
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

/// Utility functions for working with batch command results
pub mod utils {
    use super::*;
    use serde_json;
    
    /// Convert batch results to JSON format
    pub fn to_json(results: &BatchCommandResults) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(results)
    }
    
    /// Convert batch results to CSV format
    pub fn to_csv(results: &BatchCommandResults) -> String {
        let mut csv = String::new();
        
        // Add header
        csv.push_str("device_id,device_type,command,status,duration_ms,start_time,end_time\n");
        
        // Add rows
        for (_, device_results) in &results.results {
            for result in device_results {
                csv.push_str(&format!(
                    "{},{},{},{:?},{},{},{}\n",
                    result.device_id,
                    result.device_type,
                    result.command.replace(",", "\\,"),
                    result.status,
                    result.duration_ms,
                    result.start_time,
                    result.end_time
                ));
            }
        }
        
        csv
    }
    
    /// Group results by command
    pub fn group_by_command(results: &BatchCommandResults) -> HashMap<&str, Vec<&CommandResult>> {
        let mut grouped: HashMap<&str, Vec<&CommandResult>> = HashMap::new();
        
        for (_, device_results) in &results.results {
            for result in device_results {
                grouped
                    .entry(&result.command)
                    .or_insert_with(Vec::new)
                    .push(result);
            }
        }
        
        grouped
    }
    
    /// Group results by device
    pub fn group_by_device(results: &BatchCommandResults) -> HashMap<&str, Vec<&CommandResult>> {
        let mut grouped: HashMap<&str, Vec<&CommandResult>> = HashMap::new();
        
        for (device_id, device_results) in &results.results {
            for result in device_results {
                grouped
                    .entry(device_id)
                    .or_insert_with(Vec::new)
                    .push(result);
            }
        }
        
        grouped
    }
    
    /// Compare outputs across devices for the same command
    pub fn compare_outputs(results: &BatchCommandResults, command: &str) -> HashMap<String, Vec<String>> {
        let mut output_groups: HashMap<String, Vec<String>> = HashMap::new();
        
        for result in results.get_command_results(command) {
            if result.status == CommandStatus::Success {
                output_groups
                    .entry(result.output.clone())
                    .or_insert_with(Vec::new)
                    .push(result.device_id.clone());
            }
        }
        
        output_groups
    }
    
    /// Format results as a table
    pub fn format_as_table(results: &BatchCommandResults) -> String {
        let mut table = String::new();
        
        // Add header
        table.push_str("+-----------------+-----------------+--------------------------------+--------+-----------+------------------+\n");
        table.push_str("| Device          | Type            | Command                        | Status | Duration  | Error            |\n");
        table.push_str("+-----------------+-----------------+--------------------------------+--------+-----------+------------------+\n");
        
        // Add rows
        for (_, device_results) in &results.results {
            for result in device_results {
                let status = match result.status {
                    CommandStatus::Success => "SUCCESS",
                    CommandStatus::Failed => "FAILED",
                    CommandStatus::Timeout => "TIMEOUT",
                    CommandStatus::Skipped => "SKIPPED",
                };
                
                // Truncate long values for table display
                let device = truncate(&result.device_id, 15);
                let device_type = truncate(&result.device_type, 15);
                let command = truncate(&result.command, 30);
                let error = match &result.error {
                    Some(err) => truncate(err, 16),
                    None => "".to_string(),
                };
                
                table.push_str(&format!(
                    "| {:<15} | {:<15} | {:<30} | {:<6} | {:<9} | {:<16} |\n",
                    device,
                    device_type,
                    command,
                    status,
                    format!("{} ms", result.duration_ms),
                    error
                ));
            }
        }
        
        // Add footer
        table.push_str("+-----------------+-----------------+--------------------------------+--------+-----------+------------------+\n");
        
        // Add summary
        table.push_str(&format!(
            "Summary: {} devices, {} commands ({} success, {} failed, {} timeout, {} skipped) in {} ms\n",
            results.device_count,
            results.command_count,
            results.success_count,
            results.failure_count,
            results.timeout_count,
            results.skipped_count,
            results.duration_ms
        ));
        
        table
    }
    
    // Helper function to truncate strings for table display
    fn truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[0..max_len-3])
        }
    }
} 