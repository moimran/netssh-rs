/// Environment Configuration Example - Load config from environment variables
///
/// Set these environment variables:
/// export DEVICE_HOST=192.168.1.25
/// export DEVICE_USER=admin
/// export DEVICE_PASS=moimran@123
/// export DEVICE_SECRET=moimran@123

use netssh_core::{DeviceConfig, DeviceFactory, NetworkDeviceConnection};
use std::env;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    netssh_core::init_logging("warn", false, None, None)?;

    let config = load_config_from_env()?;
    let mut device = DeviceFactory::create_device(&config)?;
    device.connect()?;

    println!("=== SHOW VERSION ===");
    match device.send_command("show version").execute() {
        Ok(output) => println!("{}", output),
        Err(e) => eprintln!("Error: {}", e),
    }

    device.close()?;
    Ok(())
}

fn load_config_from_env() -> Result<DeviceConfig, Box<dyn std::error::Error>> {
    let host = env::var("DEVICE_HOST")
        .map_err(|_| "DEVICE_HOST environment variable is required")?;
    let username = env::var("DEVICE_USER")
        .map_err(|_| "DEVICE_USER environment variable is required")?;
    let password = env::var("DEVICE_PASS")
        .map_err(|_| "DEVICE_PASS environment variable is required")?;
    let secret = env::var("DEVICE_SECRET").ok();

    Ok(DeviceConfig {
        device_type: "cisco_ios".to_string(),
        host,
        username,
        password: Some(password),
        port: Some(22),
        timeout: Some(Duration::from_secs(30)),
        secret,
        session_log: Some("logs/env_config.log".to_string()),
    })
}
