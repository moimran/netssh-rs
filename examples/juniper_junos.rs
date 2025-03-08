use netssh_rs::error::NetsshError;
use netssh_rs::vendors::juniper::{JuniperDeviceConfig, JuniperJunosDevice};
use std::time::Duration;

fn main() -> Result<(), NetsshError> {
    // Initialize logging
    env_logger::init();

    // Create a device configuration
    let config = JuniperDeviceConfig {
        host: "192.168.1.50".to_string(),
        username: "admin".to_string(),
        password: Some("juniper123".to_string()),
        port: Some(22),
        timeout: Some(Duration::from_secs(60)),
        session_log: Some("logs/juniper_session.log".to_string()),
    };

    // Create a new JunOS device
    let mut device = JuniperJunosDevice::new(config)?;

    // Connect to the device
    println!("Connecting to Juniper JunOS device at 192.168.1.50");
    device.connect()?;

    // Check if we're in configuration mode
    let in_config = device.check_config_mode()?;
    println!("Device is in configuration mode: {}", in_config);

    // Send some show commands
    let show_commands = vec!["show version", "show interfaces terse"];
    println!("Sending show commands: {:?}", show_commands);

    for cmd in show_commands {
        println!("Sending command: {}", cmd);
        let output = device.send_command(cmd)?;
        println!("\nOutput of '{}':\n{}", cmd, output);
    }

    // Enter configuration mode
    println!("Entering config mode");
    device.config_mode(None)?;

    // Send some configuration commands
    let config_commands = vec![
        "set system host-name JuniperTest",
        "set interfaces ge-0/0/0 description \"Configured by Netssh-rs\"",
    ];
    println!("Sending config commands: {:?}", config_commands);

    for cmd in config_commands {
        println!("Sending config command: {}", cmd);
        let output = device.send_command(cmd)?;
        println!("\nOutput of '{}':\n{}", cmd, output);
    }

    // Commit the configuration
    println!("Committing configuration");
    device.commit_config()?;

    // Exit configuration mode
    println!("Exiting config mode");
    device.exit_config_mode(None)?;

    // Verify the configuration
    println!("Verifying configuration: show configuration interfaces ge-0/0/0");
    let output = device.send_command("show configuration interfaces ge-0/0/0")?;
    println!("\nVerification output:\n{}", output);

    // Disconnect from the device
    println!("Disconnecting from device");
    device.close()?;
    println!("Connection closed successfully");

    Ok(())
}
