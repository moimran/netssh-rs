// This is an example file showing how to create vendor-specific tests
// To add a new vendor:
// 1. Create a directory under vendors/ for the new vendor
// 2. Add the vendor to vendors/mod.rs
// 3. Create test files following this pattern

use netssh_core::{
    device_connection::{DeviceConfig, NetworkDeviceConnection},
    device_factory::DeviceFactory,
    error::NetsshError,
};
use std::env;
use std::time::Duration;

/// Example vendor-specific test file
/// Run with: cargo test --test vendors::example_vendor_test -- --nocapture
/// Or for mock only: MOCK_TESTS=1 cargo test --test vendors::example_vendor_test -- --nocapture

// Helper function to determine if we should run with real devices or mocks
fn use_mock_devices() -> bool {
    match env::var("MOCK_TESTS") {
        Ok(val) => val == "1",
        Err(_) => false,
    }
}

// Get environment variable with fallback
fn get_env_or(name: &str, default: &str) -> String {
    match env::var(name) {
        Ok(val) => val,
        Err(_) => default.to_string(),
    }
}

/// Sets up the device configuration based on environment variables or defaults
fn setup_device_config() -> DeviceConfig {
    let host = get_env_or("EXAMPLE_HOST", "127.0.0.1");
    let username = get_env_or("EXAMPLE_USERNAME", "admin");
    let password = get_env_or("EXAMPLE_PASSWORD", "admin");
    let port = get_env_or("EXAMPLE_PORT", "22").parse().unwrap_or(22);
    let timeout = get_env_or("EXAMPLE_TIMEOUT", "10").parse().unwrap_or(10.0);

    DeviceConfig {
        device_type: "example_vendor".to_string(),
        host,
        username,
        password: Some(password),
        port: Some(port),
        timeout: Some(Duration::from_secs_f64(timeout)),
        secret: None,
        session_log: None,
    }
}

// Create a mock module for the vendor
#[cfg(test)]
mod mock {
    use super::*;
    use std::sync::{Arc, Mutex};

    // Import the mock device
    // When creating a new vendor module, update this path based on your directory structure
    mod mock_device_mod {
        include!("../utils/mock_device.rs");
    }
    use mock_device_mod::{MockNetworkDevice, PromptStyle};

    pub struct ExampleVendorMockDevice {
        device: Arc<Mutex<MockNetworkDevice>>,
    }

    impl ExampleVendorMockDevice {
        pub fn new(username: &str, password: &str) -> Self {
            let mut device = MockNetworkDevice::new();

            // Configure the mock device
            device
                .set_device_type("example_vendor")
                .set_hostname("example-device")
                .set_prompt_style(PromptStyle::Custom("example-device# ".to_string()))
                .add_auth_credentials(username, password);

            // Add command responses
            device.add_command_response("show version", "Example Vendor OS Version 1.0\r\nModel: Example-1000\r\nUptime: 10 days\r\nexample-device# ");

            // Start the mock device
            device.start().expect("Failed to start mock device");

            Self {
                device: Arc::new(Mutex::new(device)),
            }
        }

        pub fn port(&self) -> u16 {
            self.device.lock().unwrap().port()
        }
    }

    impl Drop for ExampleVendorMockDevice {
        fn drop(&mut self) {
            if let Ok(mut device) = self.device.lock() {
                let _ = device.stop();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock::ExampleVendorMockDevice;
    use super::*;

    // Test basic functionality
    #[test]
    fn test_example_vendor_connect() -> Result<(), NetsshError> {
        if use_mock_devices() {
            // Use mock device
            let mock_device = ExampleVendorMockDevice::new("admin", "admin");
            let port = mock_device.port();

            let mut config = setup_device_config();
            config.host = "127.0.0.1".to_string();
            config.port = Some(port);

            // For real implementation:
            // let mut device = DeviceFactory::create_device(&config)?;
            // device.connect()?;
            // device.close()?;

            // For example purposes, we'll just print a message
            println!("Would connect to mock device on port {}", port);
            Ok(())
        } else {
            // For real devices, we would do something like:
            // let config = setup_device_config();
            // let mut device = DeviceFactory::create_device(&config)?;
            // device.connect()?;
            // device.close()?;

            // For example purposes, just print a message
            println!("Would connect to a real device");
            Ok(())
        }
    }
}
