pub mod controllers;
pub mod models;
pub mod routes;

/// API server configuration
pub struct ApiServerConfig {
    /// The address to bind to
    pub address: String,
    /// The port to listen on
    pub port: u16,
}

impl Default for ApiServerConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".to_string(),
            port: 8080,
        }
    }
}