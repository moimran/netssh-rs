use crate::autodetect::SSHDetect;
use crate::base_connection::BaseConnection;
use crate::device_connection::{DeviceConfig, DeviceType, NetworkDeviceConnection};
use crate::error::NetsshError;
use crate::vendors::cisco::{
    CiscoAsaDevice, CiscoDeviceConfig, CiscoIosDevice, CiscoNxosDevice, CiscoXrDevice,
};
use crate::vendors::juniper::{JuniperDeviceConfig, JuniperJunosDevice};
use std::str::FromStr;
use tracing::{debug, info, instrument, warn};

/// Factory for creating network device connections
pub struct DeviceFactory;

impl DeviceFactory {
    /// Create a device connection based on the provided configuration
    #[instrument(skip(config), fields(device_type = ?config.device_type, host = %config.host), level = "debug")]
    pub fn create_device(
        config: &DeviceConfig,
    ) -> Result<Box<dyn NetworkDeviceConnection + Send>, NetsshError> {
        debug!(
            "Creating device connection for {} of type {:?}",
            config.host, config.device_type
        );

        // Handle autodetection if device_type is "autodetect"
        if config.device_type.is_empty() || config.device_type == "autodetect" {
            info!(
                "No device type specified or autodetect requested for host {}",
                config.host
            );

            // Create a config for autodetection
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
                }
                Ok(None) => {
                    let err_msg = "Could not autodetect device type".to_string();
                    warn!("{}", err_msg);
                    return Err(NetsshError::UnsupportedOperation(err_msg));
                }
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
                let mut base_connection = BaseConnection::new()?;

                // Set the device type in BaseConnection
                base_connection.set_device_type(DeviceType::CiscoIos);

                // Create the device with the base connection and config
                let device = CiscoIosDevice::with_connection(base_connection, cisco_config);
                Ok(Box::new(device))
            }
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
                let mut base_connection = BaseConnection::new()?;

                // Set the device type in BaseConnection
                base_connection.set_device_type(DeviceType::CiscoXr);

                // Create the device with the base connection and config
                let device = CiscoXrDevice::with_connection(base_connection, cisco_config);
                Ok(Box::new(device))
            }
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
                let mut base_connection = BaseConnection::new()?;

                // Set the device type in BaseConnection
                base_connection.set_device_type(DeviceType::CiscoNxos);

                // Create the device with the base connection and config
                let device = CiscoNxosDevice::with_connection(base_connection, cisco_config);
                Ok(Box::new(device))
            }
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
                let mut base_connection = BaseConnection::new()?;

                // Set the device type in BaseConnection
                base_connection.set_device_type(DeviceType::CiscoAsa);

                // Create the device with the base connection and config
                let device = CiscoAsaDevice::with_connection(base_connection, cisco_config);
                Ok(Box::new(device))
            }
            "juniper_junos" => {
                let juniper_config = JuniperDeviceConfig {
                    host: config.host.clone(),
                    username: config.username.clone(),
                    password: config.password.clone(),
                    port: config.port,
                    timeout: config.timeout,
                    secret: config.secret.clone(),
                    session_log: config.session_log.clone(),
                };

                // Create a base connection with default config
                let mut base_connection = BaseConnection::new()?;

                // Set the device type in BaseConnection
                base_connection.set_device_type(DeviceType::JuniperJunos);

                // Create the device with the base connection and config
                let device = JuniperJunosDevice::with_connection(base_connection, juniper_config);

                Ok(Box::new(device))
            }
            _ => Err(NetsshError::UnsupportedOperation(format!(
                "Unsupported device type: {}",
                config.device_type
            ))),
        }
    }

    /// Creates a device connection using auto-detection
    ///
    /// # Arguments
    ///
    /// * `device_config` - Configuration for the device connection
    ///
    /// # Returns
    ///
    /// A boxed device connection implementing the NetworkDeviceConnection trait
    #[instrument(skip(device_config), fields(host = %device_config.host), level = "debug")]
    pub fn create_device_with_autodetect(
        device_config: &DeviceConfig,
    ) -> Result<Box<dyn NetworkDeviceConnection + Send>, NetsshError> {
        info!(
            "Attempting to create device with auto-detection for {}",
            device_config.host
        );

        // This is a placeholder for future implementation of auto-detection
        // For now, we'll default to the specified device type, or Cisco IOS if unknown
        Self::create_device(device_config)
    }

    /// Parses a device type string to the corresponding DeviceType enum
    ///
    /// # Arguments
    ///
    /// * `device_type_str` - String representation of the device type
    ///
    /// # Returns
    ///
    /// The corresponding DeviceType enum value
    #[instrument(level = "debug")]
    pub fn parse_device_type(device_type_str: &str) -> DeviceType {
        let device_type = DeviceType::from_str(device_type_str).unwrap_or_else(|_| {
            warn!(
                "Unknown device type string: '{}', defaulting to Unknown",
                device_type_str
            );
            DeviceType::Unknown
        });

        debug!(
            "Parsed device type '{}' to {:?}",
            device_type_str, device_type
        );
        device_type
    }
}
