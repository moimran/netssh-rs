/// Multiple Commands Example - Execute several commands and show outputs
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
        session_log: Some("logs/multiple_commands.log".to_string()),
    };

    let mut device = DeviceFactory::create_device(&config)?;
    device.connect()?;

    let commands = vec![
        "show version",
        "show ip interface brief",
        "show running-config | include hostname",
        "show clock",
        "show memory summary",
    ];

    for (i, command) in commands.iter().enumerate() {
        println!("=== COMMAND {} - {} ===", i + 1, command.to_uppercase());
        match device.send_command(command).execute() {
            Ok(output) => println!("{}", output),
            Err(e) => eprintln!("Error: {}", e),
        }
        if i < commands.len() - 1 {
            println!();
        }
    }

    device.close()?;
    Ok(())
}


