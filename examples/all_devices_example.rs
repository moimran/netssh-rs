use netssh_rs::{
    DeviceConfig, DeviceFactory, DeviceService, NetworkDeviceConnection, NetsshError,
    CiscoIosDevice, CiscoXrDevice, CiscoNxosDevice, CiscoAsaDevice, JuniperJunosDevice
};

fn main() -> Result<(), NetsshError> {
    // Initialize logging
    netssh_rs::initialize_logging(true, true)?;

    println!("Network Device Abstraction Example");
    println!("=================================");
    println!("This example demonstrates the use of the NetworkDeviceConnection trait");
    println!("with different device types.\n");

    // Create device configurations
    let configs = vec![
        ("Cisco IOS", DeviceConfig {
            device_type: "cisco_ios".to_string(),
            host: "192.168.1.1".to_string(),
            username: "admin".to_string(),
            password: Some("password123".to_string()),
            port: Some(22),
            timeout: Some(std::time::Duration::from_secs(60)),
            secret: Some("enable_secret".to_string()),
            session_log: Some("logs/cisco_ios.log".to_string()),
        }),
        ("Cisco XR", DeviceConfig {
            device_type: "cisco_xr".to_string(),
            host: "192.168.1.2".to_string(),
            username: "admin".to_string(),
            password: Some("password123".to_string()),
            port: Some(22),
            timeout: Some(std::time::Duration::from_secs(60)),
            secret: Some("enable_secret".to_string()),
            session_log: Some("logs/cisco_xr.log".to_string()),
        }),
        ("Cisco NX-OS", DeviceConfig {
            device_type: "cisco_nxos".to_string(),
            host: "192.168.1.3".to_string(),
            username: "admin".to_string(),
            password: Some("password123".to_string()),
            port: Some(22),
            timeout: Some(std::time::Duration::from_secs(60)),
            secret: Some("enable_secret".to_string()),
            session_log: Some("logs/cisco_nxos.log".to_string()),
        }),
        ("Cisco ASA", DeviceConfig {
            device_type: "cisco_asa".to_string(),
            host: "192.168.1.4".to_string(),
            username: "admin".to_string(),
            password: Some("password123".to_string()),
            port: Some(22),
            timeout: Some(std::time::Duration::from_secs(60)),
            secret: Some("enable_secret".to_string()),
            session_log: Some("logs/cisco_asa.log".to_string()),
        }),
        ("Juniper JunOS", DeviceConfig {
            device_type: "juniper_junos".to_string(),
            host: "192.168.1.5".to_string(),
            username: "admin".to_string(),
            password: Some("password123".to_string()),
            port: Some(22),
            timeout: Some(std::time::Duration::from_secs(60)),
            secret: None,
            session_log: Some("logs/juniper_junos.log".to_string()),
        }),
    ];

    // Demonstrate creating devices using the factory
    println!("Creating devices using DeviceFactory:");
    for (name, config) in &configs {
        println!("  - {}: {}", name, config.device_type);
        
        // In a real application, you would connect to the device here
        // For demonstration purposes, we'll just create the device object
        match DeviceFactory::create_device(config) {
            Ok(_) => println!("    Successfully created device"),
            Err(e) => println!("    Error creating device: {}", e),
        }
    }

    println!("\nDemonstrating device-specific implementations:");
    
    // Cisco IOS example
    println!("\nCisco IOS Device:");
    let ios_config = configs[0].1.clone();
    let cisco_config = netssh_rs::vendors::cisco::CiscoDeviceConfig {
        host: ios_config.host.clone(),
        username: ios_config.username.clone(),
        password: ios_config.password.clone(),
        port: ios_config.port,
        timeout: ios_config.timeout,
        secret: ios_config.secret.clone(),
        session_log: ios_config.session_log.clone(),
    };
    let base_connection = netssh_rs::BaseConnection::new()?;
    let ios_device = CiscoIosDevice::with_connection(base_connection, cisco_config);
    println!("  Device type: {}", ios_device.get_device_type());
    
    // Cisco XR example
    println!("\nCisco XR Device:");
    let xr_config = configs[1].1.clone();
    let cisco_config = netssh_rs::vendors::cisco::CiscoDeviceConfig {
        host: xr_config.host.clone(),
        username: xr_config.username.clone(),
        password: xr_config.password.clone(),
        port: xr_config.port,
        timeout: xr_config.timeout,
        secret: xr_config.secret.clone(),
        session_log: xr_config.session_log.clone(),
    };
    let base_connection = netssh_rs::BaseConnection::new()?;
    let xr_device = CiscoXrDevice::with_connection(base_connection, cisco_config);
    println!("  Device type: {}", xr_device.get_device_type());
    
    // Cisco NX-OS example
    println!("\nCisco NX-OS Device:");
    let nxos_config = configs[2].1.clone();
    let cisco_config = netssh_rs::vendors::cisco::CiscoDeviceConfig {
        host: nxos_config.host.clone(),
        username: nxos_config.username.clone(),
        password: nxos_config.password.clone(),
        port: nxos_config.port,
        timeout: nxos_config.timeout,
        secret: nxos_config.secret.clone(),
        session_log: nxos_config.session_log.clone(),
    };
    let base_connection = netssh_rs::BaseConnection::new()?;
    let nxos_device = CiscoNxosDevice::with_connection(base_connection, cisco_config);
    println!("  Device type: {}", nxos_device.get_device_type());
    
    // Cisco ASA example
    println!("\nCisco ASA Device:");
    let asa_config = configs[3].1.clone();
    let cisco_config = netssh_rs::vendors::cisco::CiscoDeviceConfig {
        host: asa_config.host.clone(),
        username: asa_config.username.clone(),
        password: asa_config.password.clone(),
        port: asa_config.port,
        timeout: asa_config.timeout,
        secret: asa_config.secret.clone(),
        session_log: asa_config.session_log.clone(),
    };
    let base_connection = netssh_rs::BaseConnection::new()?;
    let asa_device = CiscoAsaDevice::with_connection(base_connection, cisco_config);
    println!("  Device type: {}", asa_device.get_device_type());
    
    // Juniper JunOS example
    println!("\nJuniper JunOS Device:");
    let junos_config = configs[4].1.clone();
    let juniper_config = netssh_rs::vendors::juniper::JuniperDeviceConfig {
        host: junos_config.host.clone(),
        username: junos_config.username.clone(),
        password: junos_config.password.clone(),
        port: junos_config.port,
        timeout: junos_config.timeout,
        session_log: junos_config.session_log.clone(),
    };
    let junos_device = JuniperJunosDevice::new(juniper_config)?;
    println!("  Device type: {}", junos_device.get_device_type());
    
    println!("\nDone!");
    Ok(())
}
