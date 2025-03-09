use crate::device_connection::{DeviceConfig, NetworkDeviceConnection};
use crate::error::NetsshError;
use crate::vendors::cisco::{CiscoDeviceConfig, CiscoXrSsh, CiscoNxosSsh, CiscoIosDevice, CiscoAsaDevice};
use crate::vendors::juniper::{JuniperDeviceConfig, JuniperJunosDevice};
use crate::base_connection::BaseConnection;

/// Factory for creating network device connections
pub struct DeviceFactory;

impl DeviceFactory {
    /// Create a device connection based on the provided configuration
    pub fn create_device(config: &DeviceConfig) -> Result<Box<dyn NetworkDeviceConnection>, NetsshError> {
        match config.device_type.as_str() {
            "cisco_ios" => {
                let cisco_config = CiscoDeviceConfig {
                    host: config.host.clone(),
                    username: config.username.clone(),
                    password: config.password.clone(),
                    port: config.port,
                    timeout: config.timeout,
                    secret: config.secret.clone(),
                    session_log: config.session_log.clone(),
                };
                
                // Create a base connection with default config
                let base_connection = BaseConnection::new()?;
                
                // Create the device with the base connection and config
                let device = CiscoIosDevice::with_connection(base_connection, cisco_config);
                Ok(Box::new(device))
            },
            "cisco_xr" => {
                let cisco_config = CiscoDeviceConfig {
                    host: config.host.clone(),
                    username: config.username.clone(),
                    password: config.password.clone(),
                    port: config.port,
                    timeout: config.timeout,
                    secret: config.secret.clone(),
                    session_log: config.session_log.clone(),
                };
                
                // Create a base connection with default config
                let base_connection = BaseConnection::new()?;
                
                // Create the device with the base connection and config
                let device = CiscoXrSsh::with_connection(base_connection, cisco_config);
                Ok(Box::new(device))
            },
            "cisco_nxos" => {
                let cisco_config = CiscoDeviceConfig {
                    host: config.host.clone(),
                    username: config.username.clone(),
                    password: config.password.clone(),
                    port: config.port,
                    timeout: config.timeout,
                    secret: config.secret.clone(),
                    session_log: config.session_log.clone(),
                };
                
                // Create a base connection with default config
                let base_connection = BaseConnection::new()?;
                
                // Create the device with the base connection and config
                let device = CiscoNxosSsh::with_connection(base_connection, cisco_config);
                Ok(Box::new(device))
            },
            "cisco_asa" => {
                let cisco_config = CiscoDeviceConfig {
                    host: config.host.clone(),
                    username: config.username.clone(),
                    password: config.password.clone(),
                    port: config.port,
                    timeout: config.timeout,
                    secret: config.secret.clone(),
                    session_log: config.session_log.clone(),
                };
                
                // Create a base connection with default config
                let base_connection = BaseConnection::new()?;
                
                // Create the device with the base connection and config
                let device = CiscoAsaDevice::with_connection(base_connection, cisco_config);
                Ok(Box::new(device))
            },
            "juniper_junos" => {
                let juniper_config = JuniperDeviceConfig {
                    host: config.host.clone(),
                    username: config.username.clone(),
                    password: config.password.clone(),
                    port: config.port,
                    timeout: config.timeout,
                    session_log: config.session_log.clone(),
                };
                
                let device = JuniperJunosDevice::new(juniper_config)?;
                Ok(Box::new(device))
            },
            _ => Err(NetsshError::UnsupportedOperation(format!(
                "Unsupported device type: {}", 
                config.device_type
            ))),
        }
    }
}
