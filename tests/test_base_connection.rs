mod common;

use netssh_rs::BaseConnection;
use common::{setup_logging, MockDevice, DeviceType, get_valid_credentials, get_invalid_credentials};

#[test]
fn test_new_connection() {
    setup_logging();
    let conn = BaseConnection::new();
    assert!(conn.is_ok());
    let conn = conn.unwrap();
    assert!(conn.session.is_none());
    assert!(conn.channel.is_none());
    assert!(conn.base_prompt.is_none());
}

#[test]
fn test_connect_success() {
    setup_logging();
    let mut conn = BaseConnection::new().unwrap();
    let mock_device = MockDevice::new(DeviceType::CiscoXr);
    let (username, password) = get_valid_credentials();
    
    // Replace the real session with our mock
    conn.session = Some(mock_device.create_mocked_session());
    
    let result = conn.connect("mock_host", &username, Some(&password), None, None);
    assert!(result.is_ok());
    
    assert!(conn.session.is_some());
    assert!(conn.channel.is_some());
}

#[test]
fn test_read_write_channel() {
    setup_logging();
    let mut conn = BaseConnection::new().unwrap();
    let mock_device = MockDevice::new(DeviceType::CiscoXr);
    let (username, password) = get_valid_credentials();
    
    // Set up mock session
    conn.session = Some(mock_device.create_mocked_session());
    conn.connect("mock_host", &username, Some(&password), None, None).unwrap();
    
    // Test write and read
    let result = conn.send_command("show version");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Cisco IOS XR Software"));
}

#[test]
fn test_authentication_failure() {
    setup_logging();
    let mut conn = BaseConnection::new().unwrap();
    let mock_device = MockDevice::new(DeviceType::CiscoXr);
    let (username, password) = get_invalid_credentials();
    
    // Set up mock session
    conn.session = Some(mock_device.create_mocked_session());
    
    let result = conn.connect("mock_host", &username, Some(&password), None, None);
    assert!(result.is_err());
}

#[test]
fn test_command_error() {
    setup_logging();
    let mut conn = BaseConnection::new().unwrap();
    let mock_device = MockDevice::new(DeviceType::CiscoXr);
    let (username, password) = get_valid_credentials();
    
    // Set up mock session
    conn.session = Some(mock_device.create_mocked_session());
    conn.connect("mock_host", &username, Some(&password), None, None).unwrap();
    
    // Test invalid command
    let result = conn.send_command("invalid_command");
    assert!(result.is_ok()); // Should still succeed but with error message
    let output = result.unwrap();
    assert!(output.contains("Unknown command"));
}
