/// Basic Connection Example - Minimal output focused on command results
use netssh_core::{CommandOutput, DeviceConfig, DeviceFactory, NetworkDeviceConnection};
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
        session_log: Some("logs/basic_connection.log".to_string()),
    };

    let mut device = DeviceFactory::create_device(&config)?;
    device.connect()?;

    // Example 1: Show version command with CommandResult
    println!("=== SHOW VERSION (with CommandResult) ===");
    match device.send_command("show version").execute_with_result() {
        Ok(result) => {
            println!("Command: {}", result.command);
            println!("Status: {:?}", result.status);
            println!("Duration: {} ms", result.duration_ms_display());
            if let Some(output) = &result.output {
                println!("Output:\n{}", output.to_display_string());
            }
        },
        Err(e) => eprintln!("Error: {}", e),
    }

    // Example 2: Show IP interface brief - traditional string output
    println!("\n=== SHOW IP INTERFACE BRIEF (traditional) ===");
    match device.send_command("show ip interface brief").execute_with_result() {
        Ok(result) => {
            println!("Command: {}", result.command);
            println!("Duration: {} ms", result.duration_ms_display());
            // print commandresult
            if let Ok(json) = result.to_json() {
                println!("{}", json);
            }
            // if let Some(output) = &result.output {
            //     println!("Output:\n{}", output.to_display_string());
            // }
        },
        Err(e) => eprintln!("Error: {}", e),
    }

    // Example 3: Show running config hostname with CommandResult
    println!("\n=== HOSTNAME CONFIG (with CommandResult) ===");
    match device.send_command("show running-config | include hostname").execute_with_result() {
        Ok(result) => {
            println!("Command: {}", result.command);
            println!("Duration: {} ms", result.duration_ms_display());
            if let Some(output) = &result.output {
                println!("Output: {}", output.to_display_string().trim());
            }
        },
        Err(e) => eprintln!("Error: {}", e),
    }

    // Example 4: Command with parsing - show version with TextFSM parsing
    println!("\n=== SHOW VERSION WITH PARSING ===");
    match device.send_command("show version").parse() {
        Ok(result) => {
            // Use the new print method for formatted output
            result.print_summary();

            if let Some(output) = &result.output {
                match output {
                    CommandOutput::Raw(raw_text) => {
                        println!("Raw Output:\n{}", raw_text);
                    }
                    CommandOutput::Parsed(parsed_data) => {
                        println!("Parsed Data ({} records):", parsed_data.len());
                        if let Ok(json) = serde_json::to_string_pretty(parsed_data) {
                            println!("{}", json);
                        }
                    }
                }
            }

            if let Some(error) = &result.parse_error {
                println!("Parse Error: {}", error);
            }
        },
        Err(e) => eprintln!("Error: {}", e),
    }

    // Example 5: Demonstrate JSON conversion capabilities
    println!("\n=== JSON CONVERSION EXAMPLES ===");
    match device.send_command("show clock").execute_with_result() {
        Ok(result) => {
            println!("--- Summary ---");
            result.print_summary();

            println!("\n--- Output as JSON ---");
            result.print_output_json();

            println!("\n--- Full CommandResult as JSON (first 500 chars) ---");
            if let Ok(json) = result.to_json() {
                let preview = if json.len() > 500 {
                    format!("{}...", &json[..500])
                } else {
                    json
                };
                println!("{}", preview);
            }
        },
        Err(e) => eprintln!("Error: {}", e),
    }

    device.close()?;
    Ok(())
}
