use netssh_rs::{
    initialize_logging,
    vendors::cisco::{asa::CiscoAsaDevice, CiscoDeviceConfig, CiscoDeviceConnection},
    NetsshError,
};
// No need for these imports

fn main() -> Result<(), NetsshError> {
    // Initialize logging with both debug and session logging enabled
    initialize_logging(true, true)?;

    // Get environment variables
    let host = "192.168.1.200";
    let username = "admin";
    let password = "arhaan@457";
    let secret = "moimran@124";
    
    // Create device configuration
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

    let base_connection = netssh_rs::base_connection::BaseConnection::with_config(custom_config)?;

    // Connect to device
    let mut device = CiscoAsaDevice::with_connection(base_connection, config);
    device.connect()?;
    device.enable()?;

    // Send some commands
    let output = device.send_command("show version")?;
    println!("Output from device: {}", output);

    Ok(())
}
