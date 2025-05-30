/// Basic Connection Example - Minimal output focused on command results
use netssh_core::{DeviceConfig, DeviceFactory, NetworkDeviceConnection};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    netssh_core::init_logging("warn", false, None, None)?;

    let config = DeviceConfig {
        device_type: "cisco_ios".to_string(),
        host: "192.168.1.25".to_string(),
        username: "admin".to_string(),
        password: Some("moimran@123".to_string()),
        port: Some(22),
        timeout: Some(Duration::from_secs(30)),
        secret: Some("moimran@123".to_string()),
        session_log: Some("logs/basic_connection.log".to_string()),
    };

    let mut device = DeviceFactory::create_device(&config)?;
    device.connect()?;

    // Example 1: Show version command - full output
    println!("=== SHOW VERSION ===");
    match device.send_command("show version").execute() {
        Ok(output) => println!("{}", output),
        Err(e) => eprintln!("Error: {}", e),
    }

    // Example 2: Show IP interface brief - full output
    println!("\n=== SHOW IP INTERFACE BRIEF ===");
    match device.send_command("show ip interface brief").execute() {
        Ok(output) => println!("{}", output),
        Err(e) => eprintln!("Error: {}", e),
    }

    // Example 3: Show running config hostname - filtered output
    println!("\n=== HOSTNAME CONFIG ===");
    match device.send_command("show running-config | include hostname").execute() {
        Ok(output) => println!("{}", output.trim()),
        Err(e) => eprintln!("Error: {}", e),
    }

    device.close()?;
    Ok(())
}
