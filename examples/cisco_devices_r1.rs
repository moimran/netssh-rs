use netssh_rs::{initialize_logging, CiscoBaseConnection};
use netssh_rs::vendors::{CiscoIosDevice, CiscoDeviceConfig};
use log::{debug, info};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with debug and session logs enabled
    initialize_logging(true, true)?;

    // Set connection parameters directly
    let host = "192.168.1.125";
    let username = "admin";
    let password = "moimran@123";
    let secret = "moimran@123";  // Enable secret password

    debug!("Connecting to Cisco IOS device at {}", host);

    // Create a custom NetsshConfig with longer timeouts
    let custom_config = netssh_rs::config::NetsshConfigBuilder::default()
        .connection_timeout(std::time::Duration::from_secs(60))
        .read_timeout(std::time::Duration::from_secs(30))
        .pattern_timeout(std::time::Duration::from_secs(60))
        .enable_session_log(true)
        .session_log_path("logs/session.log".to_string())
        .build();

    // Create a BaseConnection with the custom config
    let base_connection = netssh_rs::base_connection::BaseConnection::with_config(custom_config)?;

    // Create a device configuration
    let config = CiscoDeviceConfig {
        host: host.to_string(),
        username: username.to_string(),
        password: Some(password.to_string()),
        port: Some(22),
        timeout: Some(std::time::Duration::from_secs(60)),
        secret: Some(secret.to_string()),
        session_log: Some("logs/session.log".to_string()),
    };

    // Create a new Cisco IOS device instance with the custom connection and config
    let mut device = CiscoIosDevice::with_connection(base_connection, config);
    
    // Connect to the device
    device.connect()?;

    info!("Successfully connected toooooo {}", host);
    info!("check for  enable mode");
    // Check if we're in enable mode
    // debug!("Checking if device is in enable mode");
    // let in_enable = device.check_enable_mode()?;
    // info!("Device is in enable mode: {}", in_enable);

    // Send some show commands
    let show_commands = vec![
        "show version",
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
        "interface Ethernet0/0",
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
    let verify_cmd = "show running-config interface Ethernet0/0";
    info!("Verifying configuration: {}", verify_cmd);
    let output = device.send_command(verify_cmd)?;
    println!("\nVerification output:");
    println!("{}", output);

    // Save the configuration
    info!("Saving configuration");
    device.save_config()?;
    println!("Configuration saved successfully");

    // Gracefully close the connection
    info!("Disconnecting from device");
    device.close()?;
    println!("Connection closed successfully");

    Ok(())
}
