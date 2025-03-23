use crate::device_connection::{DeviceConfig, NetworkDeviceConnection};
use crate::device_factory::DeviceFactory;
use crate::error::NetsshError;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::Mutex;
use tokio::sync::Semaphore;
use tracing::{debug, info};

/// Maximum number of connections per device
const MAX_CONNECTIONS_PER_DEVICE: usize = 5;

/// Connection timeout in seconds
const CONNECTION_TIMEOUT: u64 = 300; // 5 minutes

/// A pooled connection that can be reused
pub struct PooledConnection {
    /// The actual device connection
    connection: Box<dyn NetworkDeviceConnection + Send>,
    /// The device ID this connection belongs to
    device_id: String,
    /// When this connection was last used
    last_used: Instant,
    /// Whether this connection is currently in use
    in_use: bool,
}

impl PooledConnection {
    /// Create a new pooled connection
    fn new(connection: Box<dyn NetworkDeviceConnection + Send>, device_id: String) -> Self {
        Self {
            connection,
            device_id,
            last_used: Instant::now(),
            in_use: false,
        }
    }

    /// Check if this connection has expired
    fn is_expired(&self) -> bool {
        self.last_used.elapsed() > Duration::from_secs(CONNECTION_TIMEOUT)
    }

    /// Mark this connection as in use
    fn mark_in_use(&mut self) {
        self.in_use = true;
        self.last_used = Instant::now();
    }

    /// Mark this connection as available
    fn mark_available(&mut self) {
        self.in_use = false;
        self.last_used = Instant::now();
    }
    
    /// Get a mutable reference to the connection
    pub fn connection(&mut self) -> &mut Box<dyn NetworkDeviceConnection + Send> {
        &mut self.connection
    }
    
    /// Get the device ID
    pub fn device_id(&self) -> &str {
        &self.device_id
    }
}

/// A pool of connections for devices
pub struct ConnectionPool {
    /// Connections organized by device ID
    connections: Mutex<HashMap<String, Vec<Arc<Mutex<PooledConnection>>>>>,
    /// Semaphores to limit concurrent connections per device
    semaphores: Mutex<HashMap<String, Arc<Semaphore>>>,
    /// Device configurations
    device_configs: Mutex<HashMap<String, DeviceConfig>>,
}

impl ConnectionPool {
    /// Create a new connection pool
    pub fn new() -> Self {
        Self {
            connections: Mutex::new(HashMap::new()),
            semaphores: Mutex::new(HashMap::new()),
            device_configs: Mutex::new(HashMap::new()),
        }
    }

    /// Register a device with the pool
    pub fn register_device(&self, device_id: String, config: DeviceConfig) {
        let mut device_configs = self.device_configs.lock();
        device_configs.insert(device_id.clone(), config);

        // Create a semaphore for this device if it doesn't exist
        let mut semaphores = self.semaphores.lock();
        if !semaphores.contains_key(&device_id) {
            semaphores.insert(device_id, Arc::new(Semaphore::new(MAX_CONNECTIONS_PER_DEVICE)));
        }
    }

    /// Acquire a connection for a device
    pub async fn acquire(&self, device_id: &str) -> Result<Arc<Mutex<PooledConnection>>, NetsshError> {
        // Check if the device is registered
        let device_config = {
            let device_configs = self.device_configs.lock();
            device_configs.get(device_id).cloned().ok_or_else(|| {
                NetsshError::DeviceError(format!("Device not registered: {}", device_id))
            })?
        };

        // Get or create the semaphore for this device
        let semaphore = {
            let semaphores = self.semaphores.lock();
            semaphores.get(device_id).cloned().ok_or_else(|| {
                NetsshError::DeviceError(format!("Device not registered: {}", device_id))
            })?
        };

        // Acquire a permit from the semaphore
        let _permit = semaphore.acquire().await.map_err(|e| {
            NetsshError::DeviceError(format!("Failed to acquire connection permit: {}", e))
        })?;

        // Try to find an available connection
        let connection = {
            let mut connections_map = self.connections.lock();
            let connections = connections_map.entry(device_id.to_string()).or_insert_with(Vec::new);

            // Find an available connection
            let mut available_connection = None;
            for conn in connections.iter() {
                let mut conn_guard = conn.lock();
                if !conn_guard.in_use && !conn_guard.is_expired() {
                    conn_guard.mark_in_use();
                    available_connection = Some(Arc::clone(conn));
                    break;
                }
            }

            available_connection
        };

        // If we found an available connection, return it
        if let Some(conn) = connection {
            debug!("Reusing existing connection for device {}", device_id);
            return Ok(conn);
        }

        // Otherwise, create a new connection
        debug!("Creating new connection for device {}", device_id);
        let mut device = DeviceFactory::create_device(&device_config)?;
        device.connect()?;
        device.session_preparation()?;

        let pooled_connection = PooledConnection::new(device, device_id.to_string());
        let pooled_connection = Arc::new(Mutex::new(pooled_connection));

        // Add the new connection to the pool
        {
            let mut connections_map = self.connections.lock();
            let connections = connections_map.entry(device_id.to_string()).or_insert_with(Vec::new);
            connections.push(Arc::clone(&pooled_connection));
        }

        // Mark the connection as in use
        {
            let mut conn_guard = pooled_connection.lock();
            conn_guard.mark_in_use();
        }

        Ok(pooled_connection)
    }

    /// Release a connection back to the pool
    pub fn release(&self, connection: Arc<Mutex<PooledConnection>>) {
        let device_id = {
            let mut conn_guard = connection.lock();
            conn_guard.mark_available();
            conn_guard.device_id.clone()
        };

        debug!("Released connection for device {}", device_id);
    }

    /// Clean up expired connections
    pub fn cleanup(&self) {
        let mut connections_map = self.connections.lock();
        for (device_id, connections) in connections_map.iter_mut() {
            // Remove expired connections
            connections.retain(|conn| {
                let conn_guard = conn.lock();
                if conn_guard.is_expired() && !conn_guard.in_use {
                    debug!("Removing expired connection for device {}", device_id);
                    false
                } else {
                    true
                }
            });
        }
    }
}

/// Manager for device connections
pub struct ConnectionManager {
    /// The connection pool
    pool: Arc<ConnectionPool>,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new() -> Self {
        Self {
            pool: Arc::new(ConnectionPool::new()),
        }
    }

    /// Get the connection pool
    pub fn pool(&self) -> Arc<ConnectionPool> {
        Arc::clone(&self.pool)
    }

    /// Register a device with the connection manager
    pub fn register_device(&self, device_id: String, config: DeviceConfig) {
        self.pool.register_device(device_id, config);
    }

    /// Get a connection for a device
    pub async fn get_connection(&self, device_id: &str) -> Result<Arc<Mutex<PooledConnection>>, NetsshError> {
        self.pool.acquire(device_id).await
    }

    /// Release a connection back to the pool
    pub fn release_connection(&self, connection: &Arc<Mutex<PooledConnection>>) {
        self.pool.release(Arc::clone(connection));
    }

    /// Start the connection manager
    pub async fn start(&self) {
        info!("Starting connection manager");
        // Start a background task to clean up expired connections
        let pool = Arc::clone(&self.pool);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(60)).await;
                pool.cleanup();
            }
        });
    }

    /// Stop the connection manager
    pub async fn stop(&self) {
        info!("Stopping connection manager");
        // Additional cleanup if needed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device_connection::DeviceConfig;
    use std::time::Duration;

    #[tokio::test]
    async fn test_connection_pool_register_device() {
        let pool = ConnectionPool::new();
        let device_id = "test_device".to_string();
        let config = DeviceConfig {
            device_type: "cisco_ios".to_string(),
            host: "192.168.1.1".to_string(),
            username: "admin".to_string(),
            password: Some("password".to_string()),
            port: Some(22),
            timeout: Some(Duration::from_secs(30)),
            secret: None,
            session_log: None,
        };

        pool.register_device(device_id.clone(), config);

        let device_configs = pool.device_configs.lock();
        assert!(device_configs.contains_key(&device_id));

        let semaphores = pool.semaphores.lock();
        assert!(semaphores.contains_key(&device_id));
    }

    // Additional tests would be added here
}