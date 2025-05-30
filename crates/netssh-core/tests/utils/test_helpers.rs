use netssh_core::{
    device_connection::{DeviceConfig},
    error::NetsshError,
};
use std::env;
use std::time::Duration;

/// Common helper functions for tests

/// Determines if tests should use mock devices based on the MOCK_TESTS environment variable
#[allow(dead_code)] // Used in integration tests
pub fn use_mock_devices() -> bool {
    match env::var("MOCK_TESTS") {
        Ok(val) => val == "1",
        Err(_) => false,
    }
}

/// Gets an environment variable value with a fallback default
#[allow(dead_code)] // Used in integration tests
pub fn get_env_or(name: &str, default: &str) -> String {
    match env::var(name) {
        Ok(val) => val,
        Err(_) => default.to_string(),
    }
}

/// Creates a device configuration for testing
#[allow(dead_code)] // Used in integration tests
pub fn create_device_config(device_type: &str, env_prefix: &str) -> DeviceConfig {
    let host = get_env_or(&format!("{}_HOST", env_prefix), "127.0.0.1");
    let username = get_env_or(&format!("{}_USERNAME", env_prefix), "admin");
    let password = get_env_or(&format!("{}_PASSWORD", env_prefix), "admin");
    let port = get_env_or(&format!("{}_PORT", env_prefix), "22").parse().unwrap_or(22);
    let timeout = get_env_or(&format!("{}_TIMEOUT", env_prefix), "10").parse().unwrap_or(10.0);

    DeviceConfig {
        device_type: device_type.to_string(),
        host,
        username,
        password: Some(password),
        port: Some(port),
        timeout: Some(Duration::from_secs_f64(timeout)),
        secret: None,
        session_log: None,
    }
}

/// Runs tests that require a real device, skipping if MOCK_TESTS is set
/// 
/// # Example
/// ```
/// let result = run_real_device_test(|| {
///     // Test code that uses a real device
///     Ok(())
/// });
/// assert!(result.is_ok());
/// ```
#[allow(dead_code)] // Used in integration tests
pub fn run_real_device_test<F, T>(test_fn: F) -> Result<T, NetsshError>
where
    F: FnOnce() -> Result<T, NetsshError>,
{
    if use_mock_devices() {
        println!("Skipping real device test in mock mode");
        return Ok(test_fn()?);
    } else {
        test_fn()
    }
}

/// Asserts that a string contains expected content
#[allow(dead_code)] // Used in integration tests
pub fn assert_output_contains(output: &str, expected: &str, message: &str) {
    assert!(
        output.contains(expected),
        "{} - Expected to find '{}' in:\n{}",
        message,
        expected,
        output
    );
} 