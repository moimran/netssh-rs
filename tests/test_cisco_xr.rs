mod common;

use netmiko_rs::CiscoXrSsh;
use common::{setup_logging, MockDevice, DeviceType, get_valid_credentials, get_invalid_credentials};

#[test]
fn test_connect() {
    setup_logging();
    let mock_device = MockDevice::new(DeviceType::CiscoXr);
    let (username, password) = get_valid_credentials();
    
    // Create XR device with mock session
    let mut xr = CiscoXrSsh::new("mock_host", &username, Some(&password), None).unwrap();
    xr.base.session = Some(mock_device.create_mocked_session());
    
    let result = xr.connect();
    assert!(result.is_ok());
    
    assert!(xr.base.session.is_some());
    assert!(xr.base.channel.is_some());
    assert!(xr.base.base_prompt.is_some());
}

#[test]
fn test_send_commands() {
    setup_logging();
    let mock_device = MockDevice::new(DeviceType::CiscoXr);
    let (username, password) = get_valid_credentials();
    
    // Create and connect XR device
    let mut xr = CiscoXrSsh::new("mock_host", &username, Some(&password), None).unwrap();
    xr.base.session = Some(mock_device.create_mocked_session());
    xr.connect().unwrap();
    
    // Test basic command
    let result = xr.send_command("show version");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Cisco IOS XR Software"));
    
    // Test command with pattern
    let result = xr.send_command_with_pattern("show version", r"[#>$]");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Cisco IOS XR Software"));
}

#[test]
fn test_config_mode() {
    setup_logging();
    let mock_device = MockDevice::new(DeviceType::CiscoXr);
    let (username, password) = get_valid_credentials();
    
    // Create and connect XR device
    let mut xr = CiscoXrSsh::new("mock_host", &username, Some(&password), None).unwrap();
    xr.base.session = Some(mock_device.create_mocked_session());
    xr.connect().unwrap();
    
    // Test entering config mode
    let result = xr.config_mode();
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Enter configuration commands"));
    
    // Test exiting config mode
    let result = xr.exit_config_mode();
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("End of configuration"));
}

#[test]
fn test_invalid_credentials() {
    setup_logging();
    let mock_device = MockDevice::new(DeviceType::CiscoXr);
    let (username, password) = get_invalid_credentials();
    
    // Create XR device with mock session
    let mut xr = CiscoXrSsh::new("mock_host", &username, Some(&password), None).unwrap();
    xr.base.session = Some(mock_device.create_mocked_session());
    
    let result = xr.connect();
    assert!(result.is_err());
}
