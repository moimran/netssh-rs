/// TextFSM Parsing Example - Shows raw output vs parsed structured data
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
        session_log: Some("logs/parsing_example.log".to_string()),
    };

    let mut device = DeviceFactory::create_device(&config)?;
    device.connect()?;

    println!("=== SHOW VERSION - RAW OUTPUT ===");
    let raw_result = device.send_command("show version").execute()?;
    println!("{}", raw_result);

    println!("\n=== SHOW VERSION - PARSED OUTPUT ===");
    let parsed_result = device.send_command("show version").parse()?;

    // print json parse data
    if let Some(output) = &parsed_result.output {
        if let netssh_core::CommandOutput::Parsed(parsed_data) = output {
            // Print JSON representation of the parsed data
            let json_output = serde_json::to_string_pretty(&parsed_data)?;
            println!("JSON Output: {}", json_output);
        }
    }

    println!("\n=== SHOW IP INTERFACE BRIEF - RAW OUTPUT ===");
    let raw_result = device.send_command("show ip interface brief").execute()?;
    println!("{}", raw_result);

    println!("\n=== SHOW IP INTERFACE BRIEF - PARSED OUTPUT ===");
    let parsed_result = device.send_command("show ip interface brief").parse()?;

    match parsed_result.parse_status {
        netssh_core::ParseStatus::Success => {
            if let Some(output) = &parsed_result.output {
                if let netssh_core::CommandOutput::Parsed(parsed_data) = output {
                    println!("Parsed {} interfaces:", parsed_data.len());
                    for (i, interface) in parsed_data.iter().enumerate() {
                        println!("Interface {}:", i + 1);
                        for (key, value) in interface {
                            println!("  {}: {}", key, value);
                        }
                        println!();
                    }
                }
            }
        }
        netssh_core::ParseStatus::NoTemplate => {
            println!("No TextFSM template found for 'show ip interface brief' on cisco_ios");
        }
        netssh_core::ParseStatus::Failed => {
            println!("Parsing failed: {:?}", parsed_result.parse_error);
        }
        netssh_core::ParseStatus::NotAttempted => {
            println!("Parsing was not attempted");
        }
    }

    device.close()?;
    Ok(())
}
