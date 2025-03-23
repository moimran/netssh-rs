pub mod api;
pub mod connection_manager;
pub mod rest_api;

// Re-export API types
pub use api::routes;
pub use connection_manager::{ConnectionManager, ConnectionPool};
pub use rest_api::{DeviceController, ConfigRepository, ApiError}; 