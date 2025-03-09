use netssh_rs::{
    ConfigRepository, DeviceConfig, DeviceController, NetsshError
};
use std::sync::Arc;

fn main() -> Result<(), NetsshError> {
    // Initialize logging
    netssh_rs::initialize_logging(true, true)?;

    // Create a configuration repository
    let config_repo = Arc::new(ConfigRepository::new());
    
    // Create a device controller
    let controller = DeviceController::new(config_repo.clone());
    
    // Register some devices
    println!("Registering devices...");
    
    // Cisco XR device
    controller.register_device(
        "router1".to_string(),
        DeviceConfig {
            device_type: "cisco_xr".to_string(),
            host: "192.168.1.1".to_string(),
            username: "admin".to_string(),
            password: Some("password123".to_string()),
            port: Some(22),
            timeout: Some(std::time::Duration::from_secs(60)),
            secret: Some("enable_secret".to_string()),
            session_log: Some("logs/router1.log".to_string()),
        },
    );
    
    // Cisco NX-OS device
    controller.register_device(
        "switch1".to_string(),
        DeviceConfig {
            device_type: "cisco_nxos".to_string(),
            host: "192.168.1.2".to_string(),
            username: "admin".to_string(),
            password: Some("password123".to_string()),
            port: Some(22),
            timeout: Some(std::time::Duration::from_secs(60)),
            secret: Some("enable_secret".to_string()),
            session_log: Some("logs/switch1.log".to_string()),
        },
    );
    
    // Juniper JunOS device
    controller.register_device(
        "router2".to_string(),
        DeviceConfig {
            device_type: "juniper_junos".to_string(),
            host: "192.168.1.3".to_string(),
            username: "admin".to_string(),
            password: Some("password123".to_string()),
            port: Some(22),
            timeout: Some(std::time::Duration::from_secs(60)),
            secret: None,
            session_log: Some("logs/router2.log".to_string()),
        },
    );
    
    // List all registered devices
    println!("Registered devices:");
    for device_id in controller.list_devices() {
        println!("  {}", device_id);
    }
    
    // Get device information
    println!("\nGetting device information for router1...");
    match controller.get_device_info("router1") {
        Ok(info) => {
            println!("Device Info:");
            println!("  Vendor: {}", info.vendor);
            println!("  Model: {}", info.model);
            println!("  OS Version: {}", info.os_version);
            println!("  Hostname: {}", info.hostname);
            println!("  Uptime: {}", info.uptime);
        },
        Err(e) => println!("Error getting device info: {}", e),
    }
    
    // Get interfaces
    println!("\nGetting interfaces for switch1...");
    match controller.get_interfaces("switch1") {
        Ok(interfaces) => {
            println!("Interfaces:");
            for interface in interfaces {
                println!("  Name: {}", interface.name);
                println!("  Status: {}", interface.status);
                if let Some(ip) = interface.ip_address {
                    println!("  IP Address: {}", ip);
                }
                if let Some(desc) = interface.description {
                    println!("  Description: {}", desc);
                }
                println!();
            }
        },
        Err(e) => println!("Error getting interfaces: {}", e),
    }
    
    // Configure an interface
    println!("\nConfiguring interface on router2...");
    match controller.configure_interface("router2", "ge-0/0/0", "Configured by REST API") {
        Ok(_) => println!("Interface configured successfully"),
        Err(e) => println!("Error configuring interface: {}", e),
    }
    
    // Execute a command
    println!("\nExecuting command on router1...");
    match controller.execute_command("router1", "show version") {
        Ok(output) => {
            println!("Command output:");
            println!("{}", output);
        },
        Err(e) => println!("Error executing command: {}", e),
    }
    
    println!("\nDone!");
    Ok(())
}
