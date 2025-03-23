pub mod base_connection;
pub mod channel;
pub mod config;
pub mod device_connection;
pub mod device_connection_impl;
pub mod device_factory;
pub mod device_service;
pub mod error;
pub mod logging;
pub mod session_log;
pub mod vendors;

// Import lazy_static for common regex patterns
#[macro_use]
extern crate lazy_static;

// Common regex patterns module
pub mod patterns {
    use regex::Regex;

    lazy_static! {
        // Common network device prompt patterns
        pub static ref PROMPT_PATTERN: Regex = Regex::new(r"[>#]$").unwrap();
        pub static ref CONFIG_PROMPT_PATTERN: Regex = Regex::new(r"\(config[^)]*\)#$").unwrap();
        
        // Common ANSI escape code pattern
        pub static ref ANSI_ESCAPE_PATTERN: Regex = Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]").unwrap();
        
        // Common line ending normalization pattern
        pub static ref CRLF_PATTERN: Regex = Regex::new(r"\r\n").unwrap();
        
        // Common patterns for parsing command outputs
        pub static ref IP_ADDRESS_PATTERN: Regex = Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap();
        pub static ref MAC_ADDRESS_PATTERN: Regex = Regex::new(r"\b([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})\b").unwrap();
        
        // Common error patterns
        pub static ref ERROR_PATTERN: Regex = Regex::new(r"(?i)error|invalid|failed|denied|timeout").unwrap();
    }
}

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