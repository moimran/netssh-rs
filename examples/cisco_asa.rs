use netssh_rs::{
    initialize_logging,
    vendors::cisco::{asa::CiscoAsaDevice, CiscoDeviceConfig},
    CiscoBaseConnection,
    NetsshError,
};
use std::env;
use std::time::Duration;

fn main() -> Result<(), NetsshError> {
    // Initialize logging with both debug and session logging enabled
    initialize_logging(true, true)?;

    // Get environment variables
    let host = env::var("DEVICE_HOST").expect("DEVICE_HOST not set");
    let username = env::var("DEVICE_USER").expect("DEVICE_USER not set");
    let password = env::var("DEVICE_PASS").expect("DEVICE_PASS not set");
    let enable_secret = env::var("DEVICE_SECRET").expect("DEVICE_SECRET not set");
    
    // Create device configuration
    let config = CiscoDeviceConfig {
        host,
        username,
        password: Some(password),
        port: None, // Use default port 22
        timeout: Some(Duration::from_secs(10)),
        secret: Some(enable_secret.clone()),
        session_log: Some(String::from("logs/cisco_asa_session.log")),
    };

    // Connect to device
    let mut device = CiscoAsaDevice::new(config)?;
    device.connect()?;
    device.enable(&enable_secret)?;

    // Send some commands
    let output = device.send_command("show version ")?;
    println!("Output from device: {}", output);

    Ok(())
}
