/// Error Handling Example - Demonstrate error handling patterns
use netssh_core::{DeviceConfig, DeviceFactory, NetworkDeviceConnection};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    netssh_core::init_logging("warn", false, None, None)?;

    // Test 1: Invalid device type
    println!("=== INVALID DEVICE TYPE ===");
    let invalid_config = DeviceConfig {
        device_type: "invalid_device".to_string(),
        host: "192.168.1.25".to_string(),
        username: "admin".to_string(),
        password: Some("password".to_string()),
        port: Some(22),
        timeout: Some(Duration::from_secs(5)),
        secret: None,
        session_log: None,
    };

    match DeviceFactory::create_device(&invalid_config) {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("Expected error: {}", e),
    }

    // Test 2: Connection timeout
    println!("\n=== CONNECTION TIMEOUT ===");
    let timeout_config = DeviceConfig {
        device_type: "cisco_ios".to_string(),
        host: "192.168.999.999".to_string(), // Invalid IP
        username: "admin".to_string(),
        password: Some("password".to_string()),
        port: Some(22),
        timeout: Some(Duration::from_secs(2)), // Short timeout
        secret: None,
        session_log: None,
    };

    match DeviceFactory::create_device(&timeout_config) {
        Ok(mut device) => {
            match device.connect() {
                Ok(_) => println!("Unexpected success"),
                Err(e) => println!("Expected connection error: {}", e),
            }
        }
        Err(e) => println!("Device creation error: {}", e),
    }

    // Test 3: Command error handling
    println!("\n=== COMMAND ERROR HANDLING ===");
    let config = DeviceConfig {
        device_type: "cisco_ios".to_string(),
        host: "192.168.1.25".to_string(),
        username: "admin".to_string(),
        password: Some("moimran@123".to_string()),
        port: Some(22),
        timeout: Some(Duration::from_secs(30)),
        secret: Some("moimran@123".to_string()),
        session_log: None,
    };

    if let Ok(mut device) = DeviceFactory::create_device(&config) {
        if device.connect().is_ok() {
            // Try an invalid command
            match device.send_command("show invalid-command").execute() {
                Ok(output) => println!("Command output: {}", output),
                Err(e) => println!("Command error: {}", e),
            }
            device.close().ok();
        }
    }

    Ok(())
}


