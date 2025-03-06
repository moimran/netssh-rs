use netssh_rs::{initialize_logging, CiscoBaseConnection};
use netssh_rs::vendors::{CiscoIosDevice, CiscoDeviceConfig};
use log::{debug, info};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with debug and session logs enabled
    initialize_logging(true, true)?;

    // Set connection parameters directly
    let host = "192.168.1.25";
    let username = "admin";
    let password = "moimran@123";
    let secret = "moimran@123";  // Enable secret password

    debug!("Connecting to Cisco IOS device at {}", host);

    // Create a device configuration
    let config = CiscoDeviceConfig {
        host: host.to_string(),
        username: username.to_string(),
        password: Some(password.to_string()),
        port: Some(22),
        timeout: Some(std::time::Duration::from_secs(30)),  // Set timeout in seconds
        secret: Some(secret.to_string()),  // Enable secret password
        session_log: Some("session.log".to_string()),  // Path to session log file
    };

    // Create a new Cisco IOS device instance with the config
    let mut device = CiscoIosDevice::new(config)?;
    
    // Connect to the device
    device.connect()?;

    info!("Successfully connected to {}", host);

    // Send some show commands
    let show_commands = vec![
        "show version",
        "show inventory",
        "show ip interface brief",
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
        "interface GigabitEthernet0/0",
        "description Configured by Netssh-rs",
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
    let verify_cmd = "show running-config interface GigabitEthernet0/0";
    info!("Verifying configuration: {}", verify_cmd);
    let output = device.send_command(verify_cmd)?;
    println!("\nVerification output:");
    println!("{}", output);

    info!("Disconnecting from device");
    // Use close method instead of disconnect if available, 
    // or just let the device drop which should close the connection
    device.close()?;  // Replace with appropriate method

    Ok(())
}
