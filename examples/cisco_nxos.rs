use log::{debug, info};
use netssh_rs::{initialize_logging, CiscoBaseConnection, CiscoNxosSsh};
use netssh_rs::vendors::cisco::CiscoDeviceConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with debug enabled
    initialize_logging(true, true)?;

    // Get connection parameters from environment variables
    let host = "192.168.1.59";
    let username = "admin";
    let password = "moimran@123";
    let secret = "moimran@123";  // Enable secret password

    debug!("Connecting to Cisco NX-OS device at {}", &host);

    let custom_config = netssh_rs::config::NetsshConfigBuilder::default()
    .connection_timeout(std::time::Duration::from_secs(60))
    .read_timeout(std::time::Duration::from_secs(30))
    .pattern_timeout(std::time::Duration::from_secs(60))
    .enable_session_log(true)
    .session_log_path("logs/session.log".to_string())
    .build();

    // Create a new Cisco NX-OS device instance with configuration
    let config = CiscoDeviceConfig {
        host: host.to_string(),
        username: username.to_string(),
        password: Some(password.to_string()),
        port: Some(22),
        timeout: Some(std::time::Duration::from_secs(60)),
        secret: Some(secret.to_string()),
        session_log: Some("logs/session.log".to_string()),
    };

    // Create a BaseConnection with the custom config
    let base_connection = netssh_rs::base_connection::BaseConnection::with_config(custom_config)?;
    
    let mut device = CiscoNxosSsh::with_connection(base_connection, config);
    device.connect()?;

    info!("Successfully connected to {}", &host);

    // Send some show commands
    let show_commands = vec![
        "show version",
        "show interface status",
        "show vlan brief",
        "show ip route",
        "show interface status",
        "show running-config",
    ];

    info!("Sending show commands: {:?}", show_commands);
    for cmd in show_commands {
        debug!("Sending command: {}", cmd);
        let output = device.send_command(cmd)?;
        println!("\nOutput of '{}':", cmd);
        println!("{}", output);
    }

    // Enter config mode and make some changes
    // info!("Entering config mode");
    // device.config_mode(None)?;

    // let config_commands = vec![
    //     "interface Ethernet1/1",
    //     "description Configured by Netssh-rs",
    //     "exit",
    // ];

    // info!("Sending config commands: {:?}", config_commands);
    // for cmd in config_commands {
    //     debug!("Sending config command: {}", cmd);
    //     let output = device.send_command(cmd)?;
    //     println!("\nOutput of '{}':", cmd);
    //     println!("{}", output);
    // }

    // // Exit config mode
    // info!("Exiting config mode");
    // device.exit_config_mode(None)?;

    // // Verify the configuration
    // let verify_cmd = "show running-config interface Ethernet1/1";
    // info!("Verifying configuration: {}", verify_cmd);
    // let output = device.send_command(verify_cmd)?;
    // println!("\nVerification output:");
    // println!("{}", output);

    // // Save the configuration
    // info!("Saving configuration");
    // device.save_config()?;

    info!("Disconnecting from device");
    device.disconnect()?;

    Ok(())
}
