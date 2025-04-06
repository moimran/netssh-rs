// Import from the utils module
use crate::utils::mock_device;

use mock_device::{MockNetworkDevice, PromptStyle};
use netssh_core::{
    device_connection::{DeviceConfig, NetworkDeviceConnection},
    device_factory::DeviceFactory,
    error::NetsshError,
};
use std::time::Duration;

fn setup_mock_device() -> MockNetworkDevice {
    let mut device = MockNetworkDevice::new();

    // Configure the mock device
    device.set_hostname("TestRouter")
          .set_prompt_style(PromptStyle::CiscoIOS)
          .add_auth_credentials("admin", "password")
          .add_command_response("show version", 
            "Cisco IOS Software, Version 15.2(4)M\nUptime: 10 days\nProcessor: test")
          .add_command_response("show interfaces", 
            "GigabitEthernet0/0\n  Hardware: Ethernet, address: 0000.0000.0001\n  Internet address: 192.168.1.1/24")
          .add_command_response("configure terminal", 
            "Enter configuration commands, one per line. End with CNTL/Z.")
          .add_command_response("hostname NewRouter", 
            "");

    // Start the mock device server
    device.start().expect("Failed to start mock device");

    device
}

pub fn test_basic_device_operations() -> Result<(), NetsshError> {
    // Setup mock device
    let mock_device = setup_mock_device();
    let port = mock_device.port();

    // Create device config
    let config = DeviceConfig {
        device_type: "cisco_ios".to_string(),
        host: "127.0.0.1".to_string(),
        username: "admin".to_string(),
        password: Some("password".to_string()),
        port: Some(port),
        timeout: Some(Duration::from_secs(5)),
        secret: None,
        session_log: None,
    };

    // Create and connect to device
    let mut device = DeviceFactory::create_device(&config)?;
    device.connect()?;

    // Test sending a command
    let version_output = device.send_command("show version")?;
    assert!(version_output.contains("Cisco IOS Software"));
    assert!(version_output.contains("Uptime: 10 days"));

    // Test sending another command
    let interfaces_output = device.send_command("show interfaces")?;
    assert!(interfaces_output.contains("GigabitEthernet0/0"));
    assert!(interfaces_output.contains("192.168.1.1/24"));

    // Test configuration mode
    device.enter_config_mode(None)?;
    assert!(device.check_config_mode()?);

    // Send config commands
    let config_commands = vec!["hostname NewRouter"];
    let responses = device.send_config_commands(&config_commands)?;
    assert!(!responses.is_empty());

    // Exit config mode
    device.exit_config_mode(None)?;
    assert!(!device.check_config_mode()?);

    // Close connection
    device.close()?;
    Ok(())
}

pub fn test_connection_timeout() -> Result<(), NetsshError> {
    // Create device config with very short timeout
    let config = DeviceConfig {
        device_type: "cisco_ios".to_string(),
        host: "192.0.2.1".to_string(), // Use an invalid/unreachable address
        username: "admin".to_string(),
        password: Some("password".to_string()),
        port: Some(22),
        timeout: Some(Duration::from_millis(100)),
        secret: None,
        session_log: None,
    };

    // Attempt to create and connect to device (should fail)
    let mut device = DeviceFactory::create_device(&config)?;
    let result = device.connect();

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, NetsshError::ConnectionError(_)));
    }

    Ok(())
}
