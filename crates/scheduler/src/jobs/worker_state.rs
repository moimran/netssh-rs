use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, info, warn};
use uuid::Uuid;

use netssh_core::{DeviceConfig, DeviceFactory, NetworkDeviceConnection};

use crate::config::FailureStrategy;
use crate::error::{Result, SchedulerError};
use crate::jobs::types::{CommandResult, SshJobPayload};

/// Worker state for managing SSH connections within a worker's lifecycle
pub struct WorkerState {
    connections: HashMap<String, Box<dyn NetworkDeviceConnection + Send>>,
    last_used: HashMap<String, Instant>,
    max_idle_time: Duration,
    max_connections: usize,
}

impl WorkerState {
    /// Create a new worker state
    pub fn new(max_idle_time: Duration, max_connections: usize) -> Self {
        Self {
            connections: HashMap::new(),
            last_used: HashMap::new(),
            max_idle_time,
            max_connections,
        }
    }

    /// Get or create a connection for the given job
    pub async fn get_or_create_connection(
        &mut self,
        job: &SshJobPayload,
    ) -> Result<&mut Box<dyn NetworkDeviceConnection + Send>> {
        let connection_key = self.create_connection_key(job);

        // Check if we have a valid existing connection
        let should_reuse = if let Some(_conn) = self.connections.get(&connection_key) {
            // For now, we'll assume the connection is valid if it exists and hasn't timed out
            // In the future, we could add a proper is_connected check to the trait
            self.last_used
                .get(&connection_key)
                .map_or(false, |t| t.elapsed() < self.max_idle_time)
        } else {
            false
        };

        if should_reuse {
            self.last_used.insert(connection_key.clone(), Instant::now());
            debug!(
                connection_key = %connection_key,
                "Reusing existing SSH connection"
            );
            return Ok(self.connections.get_mut(&connection_key).unwrap());
        }

        // Remove stale connection if it exists
        if self.connections.contains_key(&connection_key) {
            debug!(
                connection_key = %connection_key,
                "Removing stale SSH connection"
            );
            self.connections.remove(&connection_key);
            self.last_used.remove(&connection_key);
        }

        // Check if we're at the connection limit
        if self.connections.len() >= self.max_connections {
            self.cleanup_oldest_connection();
        }

        // Create new connection
        debug!(
            connection_key = %connection_key,
            host = %job.connection.host,
            "Creating new SSH connection"
        );

        let device_config = self.create_device_config(job)?;
        let mut device = DeviceFactory::create_device(&device_config).map_err(|e| {
            SchedulerError::SshConnection(format!("Failed to create device: {}", e))
        })?;

        device.connect().map_err(|e| {
            SchedulerError::SshConnection(format!("Failed to connect: {}", e))
        })?;

        info!(
            connection_key = %connection_key,
            host = %job.connection.host,
            device_type = %job.connection.device_type,
            "SSH connection established"
        );

        self.connections.insert(connection_key.clone(), device);
        self.last_used.insert(connection_key.clone(), Instant::now());

        Ok(self.connections.get_mut(&connection_key).unwrap())
    }

    /// Cleanup idle connections
    pub fn cleanup_idle_connections(&mut self) {
        let now = Instant::now();
        let expired_keys: Vec<_> = self
            .last_used
            .iter()
            .filter(|(_, &last_used)| now.duration_since(last_used) > self.max_idle_time)
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            if let Some(mut conn) = self.connections.remove(&key) {
                debug!(connection_key = %key, "Cleaning up idle SSH connection");
                let _ = conn.close(); // Best effort cleanup
            }
            self.last_used.remove(&key);
        }

        if !self.connections.is_empty() {
            debug!(
                active_connections = self.connections.len(),
                "Connection cleanup completed"
            );
        }
    }

    /// Cleanup the oldest connection to make room for a new one
    fn cleanup_oldest_connection(&mut self) {
        if let Some((oldest_key, _)) = self
            .last_used
            .iter()
            .min_by_key(|(_, &last_used)| last_used)
            .map(|(k, v)| (k.clone(), *v))
        {
            if let Some(mut conn) = self.connections.remove(&oldest_key) {
                warn!(
                    connection_key = %oldest_key,
                    "Removing oldest connection to make room for new connection"
                );
                let _ = conn.close(); // Best effort cleanup
            }
            self.last_used.remove(&oldest_key);
        }
    }

    /// Create a unique connection key for the job
    fn create_connection_key(&self, job: &SshJobPayload) -> String {
        format!(
            "{}:{}@{}:{}",
            job.connection.username,
            job.connection.device_type,
            job.connection.host,
            job.connection.port.unwrap_or(22)
        )
    }

    /// Create device config from job connection config
    fn create_device_config(&self, job: &SshJobPayload) -> Result<DeviceConfig> {
        let device_config = DeviceConfig {
            host: job.connection.host.clone(),
            username: job.connection.username.clone(),
            device_type: job.connection.device_type.clone(),
            port: Some(job.connection.port.unwrap_or(22)),
            timeout: job.connection.timeout_seconds.map(Duration::from_secs),
            password: job.connection.password.clone(),
            secret: job.connection.secret.clone(),
            session_log: None,
        };

        Ok(device_config)
    }

    /// Get connection statistics
    pub fn get_stats(&self) -> WorkerStats {
        WorkerStats {
            active_connections: self.connections.len(),
            max_connections: self.max_connections,
            connection_keys: self.connections.keys().cloned().collect(),
        }
    }
}

/// Worker statistics for monitoring
#[derive(Debug, Clone)]
pub struct WorkerStats {
    pub active_connections: usize,
    pub max_connections: usize,
    pub connection_keys: Vec<String>,
}

/// Shared worker state that can be passed to job handlers
pub type SharedWorkerState = Arc<Mutex<WorkerState>>;

/// Create a new shared worker state
pub fn create_shared_worker_state(
    max_idle_time: Duration,
    max_connections: usize,
) -> SharedWorkerState {
    Arc::new(Mutex::new(WorkerState::new(
        max_idle_time,
        max_connections,
    )))
}

/// Helper function to determine if job should abort on failure
pub fn should_abort_on_failure(
    failure_strategy: &FailureStrategy,
    failed_commands: usize,
    _current_index: usize,
) -> bool {
    match failure_strategy {
        FailureStrategy::ContinueOnFailure => false,
        FailureStrategy::AbortOnFirstFailure => failed_commands > 0,
        FailureStrategy::AbortAfterNFailures(n) => failed_commands >= *n,
    }
}

/// Create an error result for a failed command
pub fn create_error_result(command: &str, error: &SchedulerError) -> CommandResult {
    CommandResult {
        id: Uuid::new_v4(),
        command: command.to_string(),
        output: None,
        error: Some(error.to_string()),
        exit_code: Some(1),
        executed_at: chrono::Utc::now(),
        duration_ms: Some(0),
    }
}
