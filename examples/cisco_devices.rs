use netmiko_rs::{
    initialize_logging,
    vendors::cisco::{
        asa::CiscoAsaDevice,
        ios::CiscoIosDevice,
        xr::CiscoXrSsh,
        CiscoDeviceConfig,
        CiscoBaseConnection,
    },
    NetmikoError,
};
use std::env;
use std::time::Duration;
use log::{debug, info};

fn main() -> Result<(), NetmikoError> {
    // Initialize logging with both debug and session logging enabled
    initialize_logging(true , true)?;
    debug!("Logging initialized successfully");

    // Get environment variables
    let host = env::var("DEVICE_HOST").unwrap_or_else(|_| "192.168.0.8".to_string());
    let username = env::var("DEVICE_USER").unwrap_or_else(|_| "admin".to_string());
    let password = env::var("DEVICE_PASS").unwrap_or_else(|_| "password".to_string());
    let secret = env::var("DEVICE_SECRET").ok();

    // Create device configuration
    let config = CiscoDeviceConfig {
        host,
        username,
        password,
        secret,
        port: None,
        timeout: Some(Duration::from_secs(10)),
    };

    // Test each device type
    test_ios_device(config.clone())?;
    test_xr_device(config.clone())?;
    test_asa_device(config)?;

    Ok(())
}

fn test_ios_device(mut config: CiscoDeviceConfig) -> Result<(), NetmikoError> {
    info!("Testing Cisco IOS device...");
    let mut device = CiscoIosDevice::new(&config)?;
    run_common_commands(&mut device)?;
    Ok(())
}

fn test_xr_device(mut config: CiscoDeviceConfig) -> Result<(), NetmikoError> {
    info!("Testing Cisco XR device...");
    let mut device = CiscoXrSsh::new(&config.host, &config.username, Some(&config.password), config.timeout)?;
    run_common_commands(&mut device)?;
    Ok(())
}

fn test_asa_device(mut config: CiscoDeviceConfig) -> Result<(), NetmikoError> {
    info!("Testing Cisco ASA device...");
    let mut device = CiscoAsaDevice::new(&config)?;
    run_common_commands(&mut device)?;
    Ok(())
}

fn run_common_commands(device: &mut impl CiscoBaseConnection) -> Result<(), NetmikoError> {
    // Send some test commands
    info!("Sending show version...");
    let output = device.send_command("show version")?;
    debug!("show version output: {}", output);

    info!("Entering config mode...");
    device.config_mode()?;

    info!("Exiting config mode...");
    device.exit_config_mode()?;

    Ok(())
}
