use log::{debug, info};
use netssh_rs::{initialize_logging, CiscoXrSsh};
use netssh_rs::vendors::cisco::CiscoDeviceConnection;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with debug enabled
    initialize_logging(true, true)?;

    // Get connection parameters from environment variables
    let host = env::var("DEVICE_HOST").expect("DEVICE_HOST not set");
    let username = env::var("DEVICE_USER").expect("DEVICE_USER not set");
    let password = env::var("DEVICE_PASS").expect("DEVICE_PASS not set");

    debug!("Connecting to Cisco XR device at {}", host);

    // Create a new Cisco XR device instance
    let mut device = CiscoXrSsh::new()?;
    device.establish_connection(&host, &username, Some(&password), None, None)?;

    info!("Successfully connected to {}", host);

    // Send some show commands
    let show_commands = vec![
        "show version brief",
        "show platform",
        "show interfaces brief",
    ];

    info!("Sending show commands: {:?}", show_commands);
    for cmd in show_commands {
        debug!("Sending command: {}", cmd);
        let output = device.send_command(cmd)?;
        println!("\nOutput of '{}':", cmd);
        println!("{}", output);
    }

    // Enter config mode and make some changes
    info!("Entering config mode");
    device.config_mode(None)?;

    let config_commands = vec![
        "interface GigabitEthernet0/0/0/0",
        "description Configured by Netssh-rs",
        "commit",
    ];

    info!("Sending config commands: {:?}", config_commands);
    for cmd in config_commands {
        debug!("Sending config command: {}", cmd);
        let output = device.send_command(cmd)?;
        println!("\nOutput of '{}':", cmd);
        println!("{}", output);
    }

    // Exit config mode
    info!("Exiting config mode");
    device.exit_config_mode(None)?;

    // Verify the configuration
    let verify_cmd = "show running-config interface GigabitEthernet0/0/0/0";
    info!("Verifying configuration: {}", verify_cmd);
    let output = device.send_command(verify_cmd)?;
    println!("\nVerification output:");
    println!("{}", output);

    info!("Disconnecting from device");
    device.disconnect()?;

    Ok(())
}
