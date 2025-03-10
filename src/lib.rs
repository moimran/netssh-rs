pub mod api;
pub mod base_connection;
pub mod channel;
pub mod config;
pub mod connection_manager;
pub mod device_connection;
pub mod device_connection_impl;
pub mod device_factory;
pub mod device_service;
pub mod error;
pub mod logging;
pub mod rest_api;
pub mod session_log;
pub mod vendors;

// Re-export vendor modules
pub use vendors::cisco;
pub use vendors::juniper;

// Re-export core types
pub use base_connection::BaseConnection;
pub use config::{NetsshConfig, NetsshConfigBuilder};
pub use error::NetsshError;
pub use logging::init_logging as initialize_logging;

// Re-export vendor-specific types
pub use vendors::cisco::{CiscoDeviceConnection, CiscoBaseConnection, CiscoXrDevice, CiscoNxosDevice, CiscoIosDevice, CiscoAsaDevice};
pub use vendors::juniper::{JuniperDeviceConnection, JuniperBaseConnection, JuniperJunosDevice};

// Re-export new abstraction layer
pub use device_connection::{NetworkDeviceConnection, DeviceConfig, DeviceInfo};
pub use device_factory::DeviceFactory;
pub use device_service::{DeviceService, Interface};
pub use rest_api::{DeviceController, ConfigRepository, ApiError};
