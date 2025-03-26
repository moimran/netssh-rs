use crate::device_connection::{DeviceConfig, NetworkDeviceConnection};
use crate::error::NetsshError;
use crate::vendors::cisco::{CiscoDeviceConfig, CiscoXrDevice, CiscoNxosDevice, CiscoIosDevice, CiscoAsaDevice};
use crate::vendors::juniper::{JuniperDeviceConfig, JuniperJunosDevice};
use crate::base_connection::BaseConnection;
use crate::autodetect::SSHDetect;
use log::{debug, info, warn};

/// Factory for creating network device connections
pub struct DeviceFactory;

impl DeviceFactory {
    /// Create a device connection based on the provided configuration
    pub fn create_device(config: &DeviceConfig) -> Result<Box<dyn NetworkDeviceConnection + Send>, NetsshError> {
        // Handle autodetection if device_type is "autodetect"
        if config.device_type == "autodetect" {
            info!("Autodetecting device type for host: {}", config.host);
            
            // Create a modified config for autodetection
            let detect_config = DeviceConfig {
                device_type: "autodetect".to_string(),
                host: config.host.clone(),
                username: config.username.clone(),
                password: config.password.clone(),
                port: config.port,
                timeout: config.timeout,
                secret: config.secret.clone(),
                session_log: config.session_log.clone(),
            };
            
            // Create SSHDetect instance
            let mut detector = match SSHDetect::new(&detect_config) {
                Ok(d) => d,
                Err(e) => {
                    warn!("Failed to create autodetect connection: {}", e);
                    return Err(e);
                }
            };
            
            // Run autodetection
            let detected_type = match detector.autodetect() {
                Ok(Some(device_type)) => {
                    info!("Autodetected device type: {}", device_type);
                    device_type
                },
                Ok(None) => {
                    let err_msg = "Could not autodetect device type".to_string();
                    warn!("{}", err_msg);
                    return Err(NetsshError::UnsupportedOperation(err_msg));
                },
                Err(e) => {
                    warn!("Error during autodetection: {}", e);
                    return Err(e);
                }
            };
            
            // Disconnect the detector connection
            if let Err(e) = detector.disconnect() {
                warn!("Error disconnecting autodetect connection: {}", e);
                // Continue anyway since we're creating a new connection
            }
            
            // Create a new config with the detected device type
            let new_config = DeviceConfig {
                device_type: detected_type,
                host: config.host.clone(),
                username: config.username.clone(),
                password: config.password.clone(),
                port: config.port,
                timeout: config.timeout,
                secret: config.secret.clone(),
                session_log: config.session_log.clone(),
            };
            
            // Recursively call create_device with the new config (not autodetect)
            return DeviceFactory::create_device(&new_config);
        }
        
        // Handle known device types
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
                let device = CiscoXrDevice::with_connection(base_connection, cisco_config);
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
                let device = CiscoNxosDevice::with_connection(base_connection, cisco_config);
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
