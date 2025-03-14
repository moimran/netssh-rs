mod common;

use netssh_rs::{CiscoXrSsh, initialize_logging, vendors::cisco::{CiscoDeviceConfig, CiscoDeviceConnection}};
use common::{TestDevice, setup_logging};
use std::path::Path;

#[test]
fn test_debug_logging() {
    // Initialize logging with debug enabled
    initialize_logging(true, true).unwrap();
    
    let device = TestDevice::default();
    let config = CiscoDeviceConfig {
        host: device.host.clone(),
        username: device.username.clone(),
        password: Some(device.password.clone()),
        port: Some(device.port),
        timeout: None,
        secret: None,
        session_log: None,
    };
    let mut xr = CiscoXrSsh::new().unwrap();
    xr.base.config = config;
    
    // Send a command that should be logged
    xr.send_command("show version").unwrap();
    
    // Verify debug log exists and contains debug messages
    let debug_log = Path::new("logs/debug.log");
    assert!(debug_log.exists());
    
    let log_content = std::fs::read_to_string(debug_log).unwrap();
    assert!(log_content.contains("DEBUG"));
    assert!(log_content.contains("show version"));
}

#[test]
fn test_session_logging() {
    setup_logging();
    let device = TestDevice::default();
    
    let config = CiscoDeviceConfig {
        host: device.host.clone(),
        username: device.username.clone(),
        password: Some(device.password.clone()),
        port: Some(device.port),
        timeout: None,
        secret: None,
        session_log: None,
    };
    let mut xr = CiscoXrSsh::new().unwrap();
    xr.base.config = config;
    
    // Send a command that should be logged
    let command = "show version";
    xr.send_command(command).unwrap();
    
    // Verify session log exists and contains command
    let session_log = Path::new("logs/session.log");
    assert!(session_log.exists());
    
    let log_content = std::fs::read_to_string(session_log).unwrap();
    assert!(log_content.contains(command));
}

#[test]
fn test_logging_disabled() {
    // Initialize logging with debug disabled
    initialize_logging(false, false).unwrap();
    
    let device = TestDevice::default();
    let config = CiscoDeviceConfig {
        host: device.host.clone(),
        username: device.username.clone(),
        password: Some(device.password.clone()),
        port: Some(device.port),
        timeout: None,
        secret: None,
        session_log: None,
    };
    let mut xr = CiscoXrSsh::new().unwrap();
    xr.base.config = config;
    
    // Send a command
    xr.send_command("show version").unwrap();
    
    // Verify debug log doesn't contain debug messages
    let debug_log = Path::new("logs/debug.log");
    if debug_log.exists() {
        let log_content = std::fs::read_to_string(debug_log).unwrap();
        assert!(!log_content.contains("DEBUG"));
    }
}
