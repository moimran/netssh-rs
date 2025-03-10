use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json::Value;

/// Standardized JSON response format
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonResponse {
    /// Status of the operation (success, error)
    pub status: String,
    /// Response data
    pub data: Value,
    /// Optional error messages
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<String>>,
    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, Value>>,
}

impl JsonResponse {
    /// Create a new success response
    pub fn success(data: Value) -> Self {
        Self {
            status: "success".to_string(),
            data,
            errors: None,
            metadata: None,
        }
    }

    /// Create a new error response
    pub fn error(message: &str) -> Self {
        Self {
            status: "error".to_string(),
            data: Value::Null,
            errors: Some(vec![message.to_string()]),
            metadata: None,
        }
    }

    /// Add metadata to the response
    pub fn with_metadata(mut self, metadata: HashMap<String, Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Command execution request
#[derive(Debug, Serialize, Deserialize)]
pub struct CommandRequest {
    /// Device connection details
    pub device: DeviceDetails,
    /// Command to execute
    pub command: String,
}

/// Configuration command request
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigCommandRequest {
    /// Device connection details
    pub device: DeviceDetails,
    /// Commands to execute
    pub commands: Vec<String>,
}

/// Interface configuration request
#[derive(Debug, Serialize, Deserialize)]
pub struct InterfaceConfigRequest {
    /// Device connection details
    pub device: DeviceDetails,
    /// Interface name
    pub name: String,
    /// Interface description
    pub description: Option<String>,
    /// IP address
    pub ip_address: Option<String>,
    /// Subnet mask
    pub subnet_mask: Option<String>,
    /// Administrative status (up/down)
    pub admin_status: Option<String>,
}

/// Device connection details
#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceDetails {
    /// Device type (e.g., cisco_ios, cisco_xr)
    pub device_type: String,
    /// Host address
    pub host: String,
    /// Username for authentication
    pub username: String,
    /// Password for authentication
    pub password: Option<String>,
    /// SSH port (default: 22)
    pub port: Option<u16>,
    /// Connection timeout in seconds
    pub timeout: Option<u64>,
    /// Enable secret (for Cisco devices)
    pub secret: Option<String>,
    /// Session log path
    pub session_log: Option<String>,
}