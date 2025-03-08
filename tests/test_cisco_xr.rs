mod common;

use netssh_rs::{CiscoXrSsh, vendors::cisco::CiscoDeviceConnection};
use netssh_rs::vendors::cisco::CiscoDeviceConfig;
use common::{setup_logging, MockDevice, DeviceType, get_valid_credentials, get_invalid_credentials};

#[test]
fn test_connect() {
    setup_logging();
    let mock_device = MockDevice::new(DeviceType::CiscoXr);
    let (username, password) = get_valid_credentials();
    
    // Create XR device with mock session
    let mut xr = CiscoXrSsh::new().unwrap();
    
    // Set up the config
    let config = CiscoDeviceConfig {
        host: "mock_host".to_string(),
        username: username.to_string(),
        password: Some(password.to_string()),
        port: Some(22),
        timeout: None,
        secret: None,
        session_log: None,
    };
    
    // Set the session directly
    xr.base.connection.session = Some(mock_device.create_mocked_session());
    
    // Connect using the establish_connection method
    let result = xr.establish_connection(&config.host, &config.username, config.password.as_deref(), config.port, config.timeout);
    assert!(result.is_ok());
    
    assert!(xr.base.connection.session.is_some());
    assert!(xr.base.connection.channel.is_some());
    assert!(xr.base.connection.base_prompt.is_some());
}

#[test]
fn test_send_commands() {
    setup_logging();
    let mock_device = MockDevice::new(DeviceType::CiscoXr);
    let (username, password) = get_valid_credentials();
    
    // Create XR device
    let mut xr = CiscoXrSsh::new().unwrap();
    
    // Set up the config
    let config = CiscoDeviceConfig {
        host: "mock_host".to_string(),
        username: username.to_string(),
        password: Some(password.to_string()),
        port: Some(22),
        timeout: None,
        secret: None,
        session_log: None,
    };
    
    // Set the session directly
    xr.base.connection.session = Some(mock_device.create_mocked_session());
    
    // Connect using the establish_connection method
    xr.establish_connection(&config.host, &config.username, config.password.as_deref(), config.port, config.timeout).unwrap();
    
    // Test basic command
    let result = xr.send_command("show version");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Cisco IOS XR Software"));
}

#[test]
fn test_config_mode() {
    setup_logging();
    let mock_device = MockDevice::new(DeviceType::CiscoXr);
    let (username, password) = get_valid_credentials();
    
    // Create XR device
    let mut xr = CiscoXrSsh::new().unwrap();
    
    // Set up the config
    let config = CiscoDeviceConfig {
        host: "mock_host".to_string(),
        username: username.to_string(),
        password: Some(password.to_string()),
        port: Some(22),
        timeout: None,
        secret: None,
        session_log: None,
    };
    
    // Set the session directly
    xr.base.connection.session = Some(mock_device.create_mocked_session());
    
    // Connect using the establish_connection method
    xr.establish_connection(&config.host, &config.username, config.password.as_deref(), config.port, config.timeout).unwrap();
    
    // Test entering config mode
    let result = xr.config_mode(None);
    assert!(result.is_ok());
    
    // Test exiting config mode
    let result = xr.exit_config_mode(None);
    assert!(result.is_ok());
}

#[test]
fn test_invalid_credentials() {
    setup_logging();
    let mock_device = MockDevice::new(DeviceType::CiscoXr);
    let (username, password) = get_invalid_credentials();
    
    // Create XR device
    let mut xr = CiscoXrSsh::new().unwrap();
    
    // Set up the config
    let config = CiscoDeviceConfig {
        host: "mock_host".to_string(),
        username: username.to_string(),
        password: Some(password.to_string()),
        port: Some(22),
        timeout: None,
        secret: None,
        session_log: None,
    };
    
    // Set the session directly
    xr.base.connection.session = Some(mock_device.create_mocked_session());
    
    // Connect using the establish_connection method
    let result = xr.establish_connection(&config.host, &config.username, config.password.as_deref(), config.port, config.timeout);
    assert!(result.is_err());
}
