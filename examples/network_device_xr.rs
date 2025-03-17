use netssh_rs::{
    DeviceConfig, DeviceFactory, DeviceService, NetsshError
};

fn main() -> Result<(), NetsshError> {
    // Initialize logging
    netssh_rs::initialize_logging(true, true)?;

    // Create a device configuration
    let config = DeviceConfig {
        device_type: "cisco_xr".to_string(),
        host: "192.168.1.129".to_string(),
        username: "admin".to_string(),
        password: Some("moimran@123".to_string()),
        port: Some(22),
        timeout: Some(std::time::Duration::from_secs(60)),
        secret: Some("moimran@123".to_string()),
        session_log: Some("logs/device_session.log".to_string()),
    };

    // Create a device using the factory
    println!("Creating device...");
    let mut device = DeviceFactory::create_device(&config)?;
    
    
    // Connect to the device
    println!("Connecting to device...");
    device.connect()?;
    
    // Configure an interface
    println!("Configuring interface...");

    let sh_ver =  device.send_command("show version")?;
    println!("{}", sh_ver);

    let sh_run =  device.send_command("show run")?;
    println!("{}", sh_run);

    device.enter_config_mode(None)?;
    device.send_command("interface GigabitEthernet0/0/0/1")?;
    device.send_command("description Configured by NetworkDeviceConnection")?;

    device.save_configuration()?;

    device.exit_config_mode(None)?;

    let sh_run =  device.send_command("show run")?;
    println!("{}", sh_run);

    
    // Close the connection
    println!("Closing connection...");
    device.close()?;
    
    println!("Done!");
    Ok(())
}
