mod mock_device;

use mock_device::{MockNetworkDevice, PromptStyle};
use netssh_core::{
    BaseConnection,
    NetsshConfig,
    NetsshError,
    buffer_pool::BufferPool,
    settings::Settings,
};
use std::time::Duration;

fn setup_mock_device() -> MockNetworkDevice {
    let mut device = MockNetworkDevice::new();
    
    // Configure the mock device
    device.set_hostname("TestRouter")
          .set_prompt_style(PromptStyle::CiscoIOS)
          .add_auth_credentials("admin", "password")
          .add_command_response("show version", 
            "Cisco IOS Software, Version 15.2(4)M\nUptime: 10 days\nProcessor: test")
          .add_command_response("show interfaces", 
            "GigabitEthernet0/0\n  Hardware: Ethernet, address: 0000.0000.0001\n  Internet address: 192.168.1.1/24")
          .add_command_response("configure terminal", 
            "Enter configuration commands, one per line. End with CNTL/Z.")
          .add_command_response("hostname NewRouter", 
            "");
    
    // Start the mock device server
    device.start().expect("Failed to start mock device");
    
    device
}

#[test]
fn test_connect_and_commands() {
    // Initialize settings
    let _ = Settings::init(None);
    
    // Setup mock device
    let device = setup_mock_device();
    let port = device.port();
    
    // Create config with short timeouts for testing
    let config = NetsshConfig::builder()
        .connection_timeout(Duration::from_secs(5))
        .read_timeout(Duration::from_secs(2))
        .write_timeout(Duration::from_secs(2))
        .build();
    
    // Create a connection
    let mut conn = BaseConnection::with_config(config)
        .expect("Failed to create BaseConnection");
    
    // Connect to the mock device
    let result = conn.connect(
        &format!("127.0.0.1"), 
        "admin", 
        Some("password"), 
        Some(port), 
        None
    );
    
    assert!(result.is_ok(), "Connection failed: {:?}", result.err());
    
    // Send a command and verify the response
    let response = conn.send_command("show version").expect("Command failed");
    assert!(response.contains("Cisco IOS Software"));
    assert!(response.contains("Uptime: 10 days"));
    
    // Send another command
    let response = conn.send_command("show interfaces").expect("Command failed");
    assert!(response.contains("GigabitEthernet0/0"));
    assert!(response.contains("192.168.1.1/24"));
    
    // Test configuration mode
    let config_commands = vec![
        "configure terminal",
        "hostname NewRouter"
    ];
    
    let responses = conn.send_config_commands(&config_commands).expect("Config commands failed");
    assert!(responses[0].contains("Enter configuration commands"));
}

#[test]
fn test_connection_timeout() {
    // Setup mock device but don't start it
    let device = MockNetworkDevice::new();
    let port = device.port();
    
    // Create config with very short timeout
    let config = NetsshConfig::builder()
        .connection_timeout(Duration::from_millis(100))
        .build();
    
    // Create a connection
    let mut conn = BaseConnection::with_config(config)
        .expect("Failed to create BaseConnection");
    
    // Try to connect to the device (should fail with timeout)
    let result = conn.connect(
        &format!("127.0.0.1"), 
        "admin", 
        Some("password"), 
        Some(port), 
        None
    );
    
    assert!(result.is_err(), "Connection should have failed with timeout");
    match result {
        Err(NetsshError::ConnectionFailed(_, _)) => {
            // Expected error
        },
        Err(e) => {
            panic!("Unexpected error: {:?}", e);
        },
        Ok(_) => {
            panic!("Connection succeeded when it should have failed");
        }
    }
}

#[test]
fn test_multiple_connections() {
    // Setup mock device
    let device = setup_mock_device();
    let port = device.port();
    
    let config = NetsshConfig::builder()
        .connection_timeout(Duration::from_secs(5))
        .build();
    
    // Create multiple connections in parallel
    let mut handles = Vec::new();
    for i in 0..3 {
        let handle = std::thread::spawn(move || {
            // Create a connection
            let mut conn = BaseConnection::with_config(config.clone())
                .expect("Failed to create BaseConnection");
            
            // Connect to the device
            let result = conn.connect(
                &format!("127.0.0.1"), 
                "admin", 
                Some("password"), 
                Some(port), 
                None
            );
            
            assert!(result.is_ok(), "Connection {} failed: {:?}", i, result.err());
            
            // Send a command
            let response = conn.send_command("show version").expect("Command failed");
            assert!(response.contains("Cisco IOS Software"));
            
            // Return success
            true
        });
        
        handles.push(handle);
    }
    
    // Wait for all connections to complete
    for (i, handle) in handles.into_iter().enumerate() {
        assert!(handle.join().unwrap(), "Connection {} failed", i);
    }
}

#[test]
fn test_buffer_pool_with_connections() {
    // Get the global buffer pool
    let pool = BufferPool::global();
    
    // Setup mock device
    let device = setup_mock_device();
    let port = device.port();
    
    // Create a connection
    let mut conn1 = BaseConnection::new().expect("Failed to create BaseConnection");
    
    // Connect and send commands
    conn1.connect("127.0.0.1", "admin", Some("password"), Some(port), None)
        .expect("Connection failed");
    
    let response1 = conn1.send_command("show version").expect("Command failed");
    assert!(response1.contains("Cisco IOS Software"));
    
    // Create a second connection
    let mut conn2 = BaseConnection::new().expect("Failed to create BaseConnection");
    
    // Connect and send different commands
    conn2.connect("127.0.0.1", "admin", Some("password"), Some(port), None)
        .expect("Connection failed");
    
    let response2 = conn2.send_command("show interfaces").expect("Command failed");
    assert!(response2.contains("GigabitEthernet0/0"));
    
    // Both connections should have used buffers from the pool
    // Close the first connection
    drop(conn1);
    
    // The second connection should still work
    let response3 = conn2.send_command("show version").expect("Command failed");
    assert!(response3.contains("Cisco IOS Software"));
} 