use netssh_core::{
    vendors::cisco::{CiscoIosDevice, CiscoDeviceConfig},
    DeviceService, ParseOptions, ParseStatus,
};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    netssh_core::init_logging("info", false, None, None)?;

    // Get connection details from environment variables or use defaults
    let host = env::var("DEVICE_HOST").unwrap_or_else(|_| "192.168.1.25".to_string());
    let username = env::var("DEVICE_USERNAME").unwrap_or_else(|_| "admin".to_string());
    let password = env::var("DEVICE_PASSWORD").unwrap_or_else(|_| "moimran@123".to_string());
    let secret = env::var("DEVICE_SECRET").unwrap_or_else(|_| "moimran@123".to_string());

    println!("Testing TextFSM integration with Cisco IOS device");
    println!("Host: {}", host);
    println!("Username: {}", username);
    println!("Password: [HIDDEN]");
    println!("Secret: [HIDDEN]");
    println!();

    // Create device configuration
    let config = CiscoDeviceConfig {
        host,
        username,
        password: Some(password),
        port: Some(22),
        timeout: Some(std::time::Duration::from_secs(60)),
        secret: Some(secret),
        session_log: None,
    };

    // Create device connection
    let mut device = CiscoIosDevice::new(config)?;

    // Connect to the device
    println!("Connecting to device...");
    device.connect()?;
    println!("Connected successfully!");

    // Create device service - use Box<dyn NetworkDeviceConnection> for type erasure
    let mut service = DeviceService::new(Box::new(device) as Box<dyn netssh_core::NetworkDeviceConnection>);

    // Test commands with different parsing scenarios
    let test_commands = vec![
        ("show version", "Basic device information"),
        ("show interfaces brief", "Interface status"),
        ("show ip route", "Routing table"),
        ("show running-config", "Configuration (should have no template)"),
    ];

    for (command, description) in test_commands {
        println!("\n{}", "=".repeat(60));
        println!("Testing: {} ({})", command, description);
        println!("{}", "=".repeat(60));

        // Test 1: Execute without parsing (backward compatibility)
        println!("\n1. Testing without parsing (backward compatibility):");
        let result_no_parse = service.execute_command_with_result(command);
        println!("Status: {:?}", result_no_parse.status);
        println!("Parse Status: {:?}", result_no_parse.parse_status);
        println!("Output length: {} characters", 
                 result_no_parse.output.as_ref().map_or(0, |s| s.len()));
        
        if let Some(error) = &result_no_parse.error {
            println!("Error: {}", error);
        }

        // Test 2: Execute with parsing enabled
        println!("\n2. Testing with parsing enabled:");
        let parse_options = ParseOptions::enabled();
        let result_with_parse = service.execute_command_with_parsing(command, &parse_options);
        
        println!("Status: {:?}", result_with_parse.status);
        println!("Parse Status: {:?}", result_with_parse.parse_status);
        println!("Output length: {} characters", 
                 result_with_parse.output.as_ref().map_or(0, |s| s.len()));
        
        match result_with_parse.parse_status {
            ParseStatus::Success => {
                if let Some(parsed_data) = &result_with_parse.parsed_data {
                    println!("Parsed {} records successfully!", parsed_data.len());
                    
                    // Show first few records as example
                    for (i, record) in parsed_data.iter().take(3).enumerate() {
                        println!("Record {}: {} fields", i + 1, record.len());
                        for (key, value) in record.iter().take(5) {
                            println!("  {}: {}", key, value);
                        }
                        if record.len() > 5 {
                            println!("  ... and {} more fields", record.len() - 5);
                        }
                    }
                    
                    if parsed_data.len() > 3 {
                        println!("... and {} more records", parsed_data.len() - 3);
                    }

                    // Test JSON conversion
                    if let Some(Ok(json)) = result_with_parse.parsed_data_as_json() {
                        println!("JSON output length: {} characters", json.len());
                    }
                } else {
                    println!("No parsed data available (unexpected)");
                }
            }
            ParseStatus::Failed => {
                println!("Parsing failed: {}", 
                         result_with_parse.parse_error.as_deref().unwrap_or("Unknown error"));
            }
            ParseStatus::NoTemplate => {
                println!("No TextFSM template found for this command");
            }
            ParseStatus::NotAttempted => {
                println!("Parsing was not attempted (unexpected)");
            }
        }

        if let Some(error) = &result_with_parse.error {
            println!("Command Error: {}", error);
        }

        // Test 3: Show raw output sample (first 200 chars)
        if let Some(output) = &result_with_parse.output {
            println!("\n3. Raw output sample (first 200 chars):");
            let sample = if output.len() > 200 {
                format!("{}...", &output[..200])
            } else {
                output.clone()
            };
            println!("{}", sample);
        }
    }

    println!("\n{}", "=".repeat(60));
    println!("TextFSM Integration Test Complete!");
    println!("{}", "=".repeat(60));

    Ok(())
}
