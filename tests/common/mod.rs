use std::env;
use std::time::Duration;
use log::LevelFilter;

pub mod mock_device;
pub use mock_device::{MockDevice, DeviceType};

pub struct TestDevice {
    pub host: String,
    pub username: String,
    pub password: String,
}

impl Default for TestDevice {
    fn default() -> Self {
        Self {
            host: env::var("DEVICE_HOST").unwrap_or_else(|_| "192.168.0.8".to_string()),
            username: env::var("DEVICE_USER").unwrap_or_else(|_| "moimran".to_string()),
            password: env::var("DEVICE_PASS").unwrap_or_else(|_| "password".to_string()),
        }
    }
}

pub fn setup_logging() {
    let _ = env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .try_init();

    std::fs::create_dir_all("logs").unwrap();
}

pub fn get_test_timeout() -> Duration {
    Duration::from_secs(10)
}

pub fn get_valid_credentials() -> (String, String) {
    ("valid_user".to_string(), "valid_pass".to_string())
}

pub fn get_invalid_credentials() -> (String, String) {
    ("invalid_user".to_string(), "invalid_pass".to_string())
}
