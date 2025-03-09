use netssh_rs::{
    DeviceConfig, DeviceFactory, DeviceService, NetworkDeviceConnection, NetsshError
};

fn main() -> Result<(), NetsshError> {
    // Initialize logging
    // netssh_rs::initialize_logging(false, true)?;

    // Create a device configuration
    let config = DeviceConfig {
        device_type: "cisco_asa".to_string(),
        host: "192.168.1.59".to_string(),
        username: "admin".to_string(),
        password: Some("moimran@123".to_string()),
        port: Some(22),
        timeout: Some(std::time::Duration::from_secs(60)),
        secret: Some("moimran@123".to_string()),
        session_log: Some("logs/device_session.log".to_string()),
    };

    // Create a device using the factory
    println!("Creating device...");
    let device = DeviceFactory::create_device(&config)?;
    
    // Create a service with the device
    println!("Creating service...");
    let mut service = DeviceService::new(device);
    
    // Connect to the device
    println!("Connecting to device...");
    service.connect()?;
    
    // // Get device information
    // println!("Getting device information...");
    // let info = service.get_device_info()?;
    // println!("Device Info:");
    // println!("  Vendor: {}", info.vendor);
    // println!("  Model: {}", info.model);
    // println!("  OS Version: {}", info.os_version);
    // println!("  Hostname: {}", info.hostname);
    // println!("  Uptime: {}", info.uptime);
    
    // // Get interfaces
    // println!("Getting interfaces...");
    // let interfaces = service.get_interfaces()?;
    // println!("Interfaces:");
    // for interface in interfaces {
    //     println!("  Name: {}", interface.name);
    //     println!("  Status: {}", interface.status);
    //     if let Some(ip) = interface.ip_address {
    //         println!("  IP Address: {}", ip);
    //     }
    //     if let Some(desc) = interface.description {
    //         println!("  Description: {}", desc);
    //     }
    //     println!();
    // }
    
    // // Configure an interface
    // println!("Configuring interface...");
    // service.configure_interface("GigabitEthernet0/0/0/0", "Configured by NetworkDeviceConnection")?;
    
    // // Execute a command
    // println!("Executing command...");
    let output = service.execute_command("show running-config")?;
    println!("Command output:");
    println!("{}", output);
    
    // Close the connection
    println!("Closing connection...");
    service.close()?;
    
    println!("Done!");
    Ok(())
}
