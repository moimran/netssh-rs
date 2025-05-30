use std::sync::Arc;
use std::time::{Duration, Instant};

use apalis::prelude::*;
use chrono::Utc;
use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::config::FailureStrategy;
use crate::error::{Result, SchedulerError};
use crate::jobs::types::{CommandResult, JobResult, JobStatus, SshJobPayload};
use crate::jobs::worker_state::{create_error_result, should_abort_on_failure, SharedWorkerState, WorkerState};
use crate::storage::traits::Storage;

// Global worker state for connection reuse
static GLOBAL_WORKER_STATE: Lazy<SharedWorkerState> = Lazy::new(|| {
    Arc::new(Mutex::new(WorkerState::new(
        Duration::from_secs(300), // 5 minutes idle timeout
        10, // max 10 connections per worker
    )))
});

/// SSH job handler that executes commands on remote servers
pub async fn ssh_job_handler(
    job: SshJobPayload,
    storage: Data<Arc<dyn Storage>>,
) -> Result<JobResult> {
    let job_id = job.id;
    let start_time = Instant::now();

    info!(
        job_id = %job_id,
        host = %job.connection.host,
        device_type = %job.connection.device_type,
        command_count = job.commands.len(),
        timeout_seconds = ?job.timeout.map(|d| d.as_secs()),
        retry_count = job.retry_count.unwrap_or(0),
        "Starting SSH job execution"
    );

    // Log job start
    storage
        .log_job_message(job_id, "info", "Job started", None)
        .await
        .map_err(|e| SchedulerError::Internal(e.to_string()))?;

    // Update job status to running
    storage
        .update_job_status(job_id, JobStatus::Running)
        .await
        .map_err(|e| SchedulerError::Internal(e.to_string()))?;

    let mut job_result = JobResult {
        job_id,
        status: JobStatus::Running,
        started_at: Some(Utc::now()),
        completed_at: None,
        command_results: Vec::new(),
        error: None,
        retry_count: job.retry_count.unwrap_or(0),
    };

    // Execute SSH commands
    match execute_ssh_commands(&job).await {
        Ok(command_results) => {
            let command_count = command_results.len();
            job_result.command_results = command_results;
            job_result.status = JobStatus::Completed;
            job_result.completed_at = Some(Utc::now());

            info!(
                job_id = %job_id,
                duration_ms = start_time.elapsed().as_millis(),
                command_count = command_count,
                "SSH job completed successfully"
            );

            // Log success
            storage
                .log_job_message(
                    job_id,
                    "info",
                    &format!("Job completed successfully in {:?}", start_time.elapsed()),
                    None,
                )
                .await
                .map_err(|e| SchedulerError::Internal(e.to_string()))?;
        }
        Err(e) => {
            job_result.status = JobStatus::Failed;
            job_result.error = Some(e.to_string());
            job_result.completed_at = Some(Utc::now());

            error!(
                job_id = %job_id,
                error = %e,
                duration_ms = start_time.elapsed().as_millis(),
                "SSH job failed"
            );

            // Log failure
            storage
                .log_job_message(job_id, "error", &format!("Job failed: {}", e), None)
                .await
                .map_err(|e| SchedulerError::Internal(e.to_string()))?;
        }
    }

    // Save job result
    storage
        .save_job_result(&job_result)
        .await
        .map_err(|e| SchedulerError::Internal(e.to_string()))?;

    // Save command results
    if !job_result.command_results.is_empty() {
        storage
            .save_command_results(job_id, &job_result.command_results)
            .await
            .map_err(|e| SchedulerError::Internal(e.to_string()))?;
    }

    Ok(job_result)
}

/// Execute SSH commands using netssh-core
async fn execute_ssh_commands(job: &SshJobPayload) -> Result<Vec<CommandResult>> {
    use netssh_core::{DeviceConfig, DeviceFactory};

    let mut command_results = Vec::new();

    info!(
        host = %job.connection.host,
        device_type = %job.connection.device_type,
        username = %job.connection.username,
        port = job.connection.port.unwrap_or(22),
        timeout_seconds = job.connection.timeout_seconds.unwrap_or(30),
        has_secret = job.connection.secret.is_some(),
        "Initializing SSH connection with netssh-core"
    );

    // Create device configuration
    let device_config = DeviceConfig {
        device_type: job.connection.device_type.clone(),
        host: job.connection.host.clone(),
        username: job.connection.username.clone(),
        password: job.connection.password.clone(),
        port: job.connection.port,
        timeout: job.connection.timeout_seconds.map(Duration::from_secs),
        secret: job.connection.secret.clone(), // Enable password/secret for privileged mode
        session_log: None,                     // TODO: Add session logging support
    };

    info!(
        host = %job.connection.host,
        "Creating SSH device instance using DeviceFactory"
    );

    // Create and connect to device
    let mut device = DeviceFactory::create_device(&device_config).map_err(|e| {
        error!(
            host = %job.connection.host,
            error = %e,
            "Failed to create SSH device instance"
        );
        SchedulerError::SshConnection(e.to_string())
    })?;

    info!(
        host = %job.connection.host,
        "Attempting SSH connection..."
    );

    device.connect().map_err(|e| {
        error!(
            host = %job.connection.host,
            error = %e,
            "SSH connection failed"
        );
        SchedulerError::SshConnection(e.to_string())
    })?;

    info!(
        host = %job.connection.host,
        device_type = %job.connection.device_type,
        username = %job.connection.username,
        port = job.connection.port.unwrap_or(22),
        "SSH connection established"
    );

    // Execute each command
    for command in &job.commands {
        let start_time = Instant::now();
        let executed_at = Utc::now();

        info!(
            host = %job.connection.host,
            command = %command,
            command_index = command_results.len() + 1,
            total_commands = job.commands.len(),
            "Executing SSH command via netssh-core"
        );

        match device.send_command(command, None, None, None, None, None, None, None) {
            Ok(output) => {
                let duration = start_time.elapsed();
                let output_length = output.len();

                command_results.push(CommandResult {
                    id: Uuid::new_v4(),
                    command: command.clone(),
                    output: Some(output),
                    error: None,
                    exit_code: Some(0), // Assume success if no error
                    executed_at,
                    duration_ms: Some(duration.as_millis() as u64),
                });

                info!(
                    host = %job.connection.host,
                    command = %command,
                    duration_ms = duration.as_millis(),
                    output_length = output_length,
                    "SSH command executed successfully"
                );
            }
            Err(e) => {
                let duration = start_time.elapsed();
                let error_msg = e.to_string();

                warn!(
                    host = %job.connection.host,
                    command = %command,
                    error = %error_msg,
                    duration_ms = duration.as_millis(),
                    "SSH command failed"
                );

                command_results.push(CommandResult {
                    id: Uuid::new_v4(),
                    command: command.clone(),
                    output: None,
                    error: Some(error_msg),
                    exit_code: Some(1), // Assume failure
                    executed_at,
                    duration_ms: Some(duration.as_millis() as u64),
                });

                // Continue with other commands even if one fails
            }
        }
    }

    // Close the connection
    info!(
        host = %job.connection.host,
        "Closing SSH connection..."
    );

    device.close().map_err(|e| {
        error!(
            host = %job.connection.host,
            error = %e,
            "Failed to close SSH connection gracefully"
        );
        SchedulerError::SshConnection(e.to_string())
    })?;

    info!(
        host = %job.connection.host,
        total_commands = job.commands.len(),
        successful_commands = command_results.iter().filter(|r| r.error.is_none()).count(),
        failed_commands = command_results.iter().filter(|r| r.error.is_some()).count(),
        "SSH connection closed successfully"
    );

    Ok(command_results)
}

/// Validate SSH job payload
pub fn validate_ssh_job(payload: &SshJobPayload) -> Result<()> {
    // Validate connection details
    validate_connection_config(&payload.connection)?;

    // Validate commands
    validate_commands(&payload.commands)?;

    // Validate timeout if specified
    if let Some(timeout) = payload.timeout {
        if timeout.as_secs() == 0 {
            return Err(SchedulerError::Validation(
                "Timeout must be greater than 0 seconds".to_string(),
            ));
        }
        if timeout.as_secs() > 3600 {
            return Err(SchedulerError::Validation(
                "Timeout cannot exceed 1 hour (3600 seconds)".to_string(),
            ));
        }
    }

    // Validate retry count
    if let Some(retry_count) = payload.retry_count {
        if retry_count > 5 {
            return Err(SchedulerError::Validation(
                "Retry count cannot exceed 5".to_string(),
            ));
        }
    }

    Ok(())
}

/// Validate SSH connection configuration
fn validate_connection_config(config: &crate::jobs::types::SshConnectionConfig) -> Result<()> {
    // Validate host
    if config.host.trim().is_empty() {
        return Err(SchedulerError::Validation(
            "Host cannot be empty".to_string(),
        ));
    }

    // Validate username
    if config.username.trim().is_empty() {
        return Err(SchedulerError::Validation(
            "Username cannot be empty".to_string(),
        ));
    }

    // Validate authentication method
    if config.password.is_none() && config.private_key.is_none() {
        return Err(SchedulerError::Validation(
            "Either password or private key must be provided".to_string(),
        ));
    }

    // Validate device type
    validate_device_type(&config.device_type)?;

    // Validate port
    if let Some(port) = config.port {
        if port == 0 {
            return Err(SchedulerError::Validation(
                "Port must be between 1 and 65535".to_string(),
            ));
        }
    }

    // Validate timeout
    if let Some(timeout) = config.timeout_seconds {
        if timeout == 0 {
            return Err(SchedulerError::Validation(
                "Connection timeout must be greater than 0 seconds".to_string(),
            ));
        }
        if timeout > 300 {
            return Err(SchedulerError::Validation(
                "Connection timeout cannot exceed 300 seconds".to_string(),
            ));
        }
    }

    Ok(())
}

/// Validate device type
fn validate_device_type(device_type: &str) -> Result<()> {
    const VALID_DEVICE_TYPES: &[&str] = &[
        "cisco_ios",
        "cisco_ios_xe",
        "cisco_nxos",
        "cisco_asa",
        "cisco_xr",
        "arista_eos",
        "juniper_junos",
    ];

    if device_type.trim().is_empty() {
        return Err(SchedulerError::Validation(
            "Device type cannot be empty".to_string(),
        ));
    }

    if !VALID_DEVICE_TYPES.contains(&device_type) {
        return Err(SchedulerError::Validation(format!(
            "Invalid device type: '{}'. Valid types: {}",
            device_type,
            VALID_DEVICE_TYPES.join(", ")
        )));
    }

    Ok(())
}

/// Validate commands list
fn validate_commands(commands: &[String]) -> Result<()> {
    if commands.is_empty() {
        return Err(SchedulerError::Validation(
            "Commands list cannot be empty".to_string(),
        ));
    }

    if commands.len() > 50 {
        return Err(SchedulerError::Validation(
            "Cannot execute more than 50 commands in a single job".to_string(),
        ));
    }

    for (index, command) in commands.iter().enumerate() {
        if command.trim().is_empty() {
            return Err(SchedulerError::Validation(format!(
                "Command at index {} cannot be empty",
                index
            )));
        }

        if command.len() > 1000 {
            return Err(SchedulerError::Validation(format!(
                "Command at index {} exceeds maximum length of 1000 characters",
                index
            )));
        }
    }

    Ok(())
}

/// Enhanced SSH job handler with connection reuse and improved error handling
pub async fn enhanced_ssh_job_handler(
    job: SshJobPayload,
    storage: Data<Arc<dyn Storage>>,
    worker_state: Data<SharedWorkerState>,
    failure_strategy: FailureStrategy,
) -> Result<JobResult> {
    let job_id = job.id;
    let start_time = Instant::now();

    info!(
        job_id = %job_id,
        host = %job.connection.host,
        device_type = %job.connection.device_type,
        command_count = job.commands.len(),
        timeout_seconds = ?job.timeout.map(|d| d.as_secs()),
        retry_count = job.retry_count.unwrap_or(0),
        connection_reuse = true,
        "Starting enhanced SSH job execution"
    );

    // Log job start
    storage
        .log_job_message(job_id, "info", "Enhanced job started with connection reuse", None)
        .await
        .map_err(|e| SchedulerError::Internal(e.to_string()))?;

    // Initialize job result
    let mut job_result = JobResult {
        job_id,
        status: JobStatus::Running,
        started_at: Some(Utc::now()),
        completed_at: None,
        command_results: Vec::new(),
        error: None,
        retry_count: job.retry_count.unwrap_or(0),
    };

    // Execute SSH commands with connection reuse
    match execute_ssh_commands_with_reuse(&job, worker_state, &failure_strategy).await {
        Ok(command_results) => {
            let command_count = command_results.len();
            let successful_commands = command_results.iter().filter(|r| r.error.is_none()).count();
            let failed_commands = command_results.iter().filter(|r| r.error.is_some()).count();

            job_result.command_results = command_results;
            job_result.status = if failed_commands == 0 { JobStatus::Completed } else { JobStatus::Failed };
            job_result.completed_at = Some(Utc::now());

            info!(
                job_id = %job_id,
                duration_ms = start_time.elapsed().as_millis(),
                command_count = command_count,
                successful_commands = successful_commands,
                failed_commands = failed_commands,
                "Enhanced SSH job completed"
            );

            // Log success
            storage
                .log_job_message(
                    job_id,
                    "info",
                    &format!(
                        "Enhanced job completed in {:?} - {}/{} commands successful",
                        start_time.elapsed(),
                        successful_commands,
                        command_count
                    ),
                    None,
                )
                .await
                .map_err(|e| SchedulerError::Internal(e.to_string()))?;
        }
        Err(e) => {
            job_result.status = JobStatus::Failed;
            job_result.completed_at = Some(Utc::now());
            job_result.error = Some(e.to_string());

            error!(
                job_id = %job_id,
                duration_ms = start_time.elapsed().as_millis(),
                error = %e,
                "Enhanced SSH job failed"
            );

            // Log error
            storage
                .log_job_message(
                    job_id,
                    "error",
                    &format!("Enhanced job failed: {}", e),
                    None,
                )
                .await
                .map_err(|e| SchedulerError::Internal(e.to_string()))?;
        }
    }

    // Save job result
    storage
        .save_job_result(&job_result)
        .await
        .map_err(|e| SchedulerError::Internal(e.to_string()))?;

    Ok(job_result)
}

/// Execute SSH commands with connection reuse and enhanced error handling
async fn execute_ssh_commands_with_reuse(
    job: &SshJobPayload,
    worker_state: Data<SharedWorkerState>,
    failure_strategy: &FailureStrategy,
) -> Result<Vec<CommandResult>> {
    let mut command_results = Vec::new();
    let mut failed_commands = 0;

    info!(
        host = %job.connection.host,
        device_type = %job.connection.device_type,
        command_count = job.commands.len(),
        "Executing commands with reused connection"
    );

    // Execute commands with better error handling
    for (index, command) in job.commands.iter().enumerate() {
        let start_time = Instant::now();

        info!(
            host = %job.connection.host,
            command = %command,
            command_index = index + 1,
            total_commands = job.commands.len(),
            "Executing SSH command"
        );

        // Get connection for each command to avoid borrowing issues
        let result = {
            let mut state = worker_state.lock().await;
            let connection = state.get_or_create_connection(job).await?;
            execute_command_with_retry_sync(connection, command, job).await
        };

        match result {
            Ok(command_result) => {
                command_results.push(command_result);
                debug!(
                    host = %job.connection.host,
                    command = %command,
                    duration_ms = start_time.elapsed().as_millis(),
                    "Command executed successfully"
                );
            }
            Err(e) => {
                failed_commands += 1;
                let error_msg = e.to_string();
                let error_result = create_error_result(command, &e);
                command_results.push(error_result);

                warn!(
                    host = %job.connection.host,
                    command = %command,
                    error = %error_msg,
                    failed_commands = failed_commands,
                    "Command execution failed"
                );

                // Check if we should abort based on failure strategy
                if should_abort_on_failure(failure_strategy, failed_commands, index) {
                    warn!(
                        host = %job.connection.host,
                        failed_commands = failed_commands,
                        strategy = ?failure_strategy,
                        "Aborting job execution due to failure strategy"
                    );
                    break;
                }
            }
        }
    }

    // Cleanup idle connections periodically (every 10th job)
    if job.id.as_u128() % 10 == 0 {
        let mut state = worker_state.lock().await;
        state.cleanup_idle_connections();

        let stats = state.get_stats();
        debug!(
            active_connections = stats.active_connections,
            max_connections = stats.max_connections,
            "Connection cleanup completed"
        );
    }

    info!(
        host = %job.connection.host,
        total_commands = job.commands.len(),
        successful_commands = command_results.iter().filter(|r| r.error.is_none()).count(),
        failed_commands = command_results.iter().filter(|r| r.error.is_some()).count(),
        "Command execution completed with connection reuse"
    );

    Ok(command_results)
}

/// Execute a single command with retry logic (synchronous version for use within locks)
async fn execute_command_with_retry_sync(
    connection: &mut Box<dyn netssh_core::NetworkDeviceConnection + Send>,
    command: &str,
    job: &SshJobPayload,
) -> Result<CommandResult> {
    let executed_at = Utc::now();
    let start_time = Instant::now();
    let max_retries = job.retry_count.unwrap_or(0);

    for attempt in 0..=max_retries {
        match connection.send_command(command, None, None, None, None, None, None, None) {
            Ok(output) => {
                let duration = start_time.elapsed();
                return Ok(CommandResult {
                    id: Uuid::new_v4(),
                    command: command.to_string(),
                    output: Some(output),
                    error: None,
                    exit_code: Some(0),
                    executed_at,
                    duration_ms: Some(duration.as_millis() as u64),
                });
            }
            Err(e) => {
                if attempt < max_retries {
                    warn!(
                        command = %command,
                        attempt = attempt + 1,
                        max_retries = max_retries,
                        error = %e,
                        "Command failed, retrying..."
                    );
                    // Small delay before retry
                    tokio::time::sleep(Duration::from_millis(100 * (attempt + 1) as u64)).await;
                } else {
                    return Err(SchedulerError::SshExecution(e.to_string()));
                }
            }
        }
    }

    unreachable!("Should have returned from retry loop")
}

/// Enhanced SSH job handler that uses global worker state for connection reuse
pub async fn enhanced_ssh_job_handler_global(
    job: SshJobPayload,
    storage: Data<Arc<dyn Storage>>,
) -> Result<JobResult> {
    let job_id = job.id;
    let start_time = Instant::now();

    info!(
        job_id = %job_id,
        host = %job.connection.host,
        device_type = %job.connection.device_type,
        command_count = job.commands.len(),
        timeout_seconds = ?job.timeout.map(|d| d.as_secs()),
        retry_count = job.retry_count.unwrap_or(0),
        connection_reuse = true,
        "Starting enhanced SSH job execution with global state"
    );

    // Log job start
    storage
        .log_job_message(job_id, "info", "Enhanced job started with connection reuse (global state)", None)
        .await
        .map_err(|e| SchedulerError::Internal(e.to_string()))?;

    // Initialize job result
    let mut job_result = JobResult {
        job_id,
        status: JobStatus::Running,
        started_at: Some(Utc::now()),
        completed_at: None,
        command_results: Vec::new(),
        error: None,
        retry_count: job.retry_count.unwrap_or(0),
    };

    // Execute SSH commands with connection reuse using global state
    match execute_ssh_commands_with_global_reuse(&job, &FailureStrategy::ContinueOnFailure).await {
        Ok(command_results) => {
            let command_count = command_results.len();
            let successful_commands = command_results.iter().filter(|r| r.error.is_none()).count();
            let failed_commands = command_results.iter().filter(|r| r.error.is_some()).count();

            job_result.command_results = command_results;
            job_result.status = if failed_commands == 0 { JobStatus::Completed } else { JobStatus::Failed };
            job_result.completed_at = Some(Utc::now());

            info!(
                job_id = %job_id,
                duration_ms = start_time.elapsed().as_millis(),
                command_count = command_count,
                successful_commands = successful_commands,
                failed_commands = failed_commands,
                "Enhanced SSH job completed with global state"
            );

            // Log success
            storage
                .log_job_message(
                    job_id,
                    "info",
                    &format!(
                        "Enhanced job completed in {:?} - {}/{} commands successful",
                        start_time.elapsed(),
                        successful_commands,
                        command_count
                    ),
                    None,
                )
                .await
                .map_err(|e| SchedulerError::Internal(e.to_string()))?;
        }
        Err(e) => {
            job_result.status = JobStatus::Failed;
            job_result.completed_at = Some(Utc::now());
            job_result.error = Some(e.to_string());

            error!(
                job_id = %job_id,
                duration_ms = start_time.elapsed().as_millis(),
                error = %e,
                "Enhanced SSH job failed with global state"
            );

            // Log error
            storage
                .log_job_message(
                    job_id,
                    "error",
                    &format!("Enhanced job failed: {}", e),
                    None,
                )
                .await
                .map_err(|e| SchedulerError::Internal(e.to_string()))?;
        }
    }

    // Save job result
    storage
        .save_job_result(&job_result)
        .await
        .map_err(|e| SchedulerError::Internal(e.to_string()))?;

    Ok(job_result)
}

/// Execute SSH commands with global connection reuse
async fn execute_ssh_commands_with_global_reuse(
    job: &SshJobPayload,
    failure_strategy: &FailureStrategy,
) -> Result<Vec<CommandResult>> {
    let mut command_results = Vec::new();
    let mut failed_commands = 0;

    info!(
        host = %job.connection.host,
        device_type = %job.connection.device_type,
        command_count = job.commands.len(),
        "Executing commands with global connection reuse"
    );

    // Execute commands with better error handling
    for (index, command) in job.commands.iter().enumerate() {
        let start_time = Instant::now();

        info!(
            host = %job.connection.host,
            command = %command,
            command_index = index + 1,
            total_commands = job.commands.len(),
            "Executing SSH command with global state"
        );

        // Get connection for each command using global state
        let result = {
            let mut state = GLOBAL_WORKER_STATE.lock().await;
            let connection = state.get_or_create_connection(job).await?;
            execute_command_with_retry_sync(connection, command, job).await
        };

        match result {
            Ok(command_result) => {
                command_results.push(command_result);
                debug!(
                    host = %job.connection.host,
                    command = %command,
                    duration_ms = start_time.elapsed().as_millis(),
                    "Command executed successfully with global state"
                );
            }
            Err(e) => {
                failed_commands += 1;
                let error_msg = e.to_string();
                let error_result = create_error_result(command, &e);
                command_results.push(error_result);

                warn!(
                    host = %job.connection.host,
                    command = %command,
                    error = %error_msg,
                    failed_commands = failed_commands,
                    "Command execution failed with global state"
                );

                // Check if we should abort based on failure strategy
                if should_abort_on_failure(failure_strategy, failed_commands, index) {
                    warn!(
                        host = %job.connection.host,
                        failed_commands = failed_commands,
                        strategy = ?failure_strategy,
                        "Aborting job execution due to failure strategy"
                    );
                    break;
                }
            }
        }
    }

    // Cleanup idle connections periodically (every 10th job)
    if job.id.as_u128() % 10 == 0 {
        let mut state = GLOBAL_WORKER_STATE.lock().await;
        state.cleanup_idle_connections();

        let stats = state.get_stats();
        debug!(
            active_connections = stats.active_connections,
            max_connections = stats.max_connections,
            "Global connection cleanup completed"
        );
    }

    info!(
        host = %job.connection.host,
        total_commands = job.commands.len(),
        successful_commands = command_results.iter().filter(|r| r.error.is_none()).count(),
        failed_commands = command_results.iter().filter(|r| r.error.is_some()).count(),
        "Command execution completed with global connection reuse"
    );

    Ok(command_results)
}
