/// Device Information Example - Extract specific info from command outputs
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
        session_log: Some("logs/device_info.log".to_string()),
    };

    let mut device = DeviceFactory::create_device(&config)?;
    device.connect()?;

    // Get hostname
    println!("=== HOSTNAME ===");
    match device.send_command("show running-config | include hostname").execute() {
        Ok(output) => {
            if let Some(hostname) = extract_hostname(&output) {
                println!("Hostname: {}", hostname);
            } else {
                println!("Hostname: Not found");
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    // Get version info
    println!("\n=== VERSION INFO ===");
    match device.send_command("show version").execute() {
        Ok(output) => {
            if let Some(version) = extract_version(&output) {
                println!("Version: {}", version);
            }
            if let Some(uptime) = extract_uptime(&output) {
                println!("Uptime: {}", uptime);
            }
            if let Some(serial) = extract_serial(&output) {
                println!("Serial: {}", serial);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    // Get interface summary
    println!("\n=== INTERFACE SUMMARY ===");
    match device.send_command("show ip interface brief").execute() {
        Ok(output) => {
            let interfaces = count_interfaces(&output);
            println!("Total interfaces: {}", interfaces.0);
            println!("Active interfaces: {}", interfaces.1);
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    device.close()?;
    Ok(())
}

// Simple extraction functions
fn extract_hostname(output: &str) -> Option<String> {
    for line in output.lines() {
        if line.contains("hostname") {
            if let Some(hostname) = line.split_whitespace().nth(1) {
                return Some(hostname.to_string());
            }
        }
    }
    None
}

fn extract_version(output: &str) -> Option<String> {
    for line in output.lines() {
        let line = line.trim();
        if line.contains("Version") && (line.contains("IOS") || line.contains("NX-OS") || line.contains("ASA")) {
            return Some(line.to_string());
        }
    }
    None
}

fn extract_uptime(output: &str) -> Option<String> {
    for line in output.lines() {
        let line = line.trim();
        if line.contains("uptime") {
            return Some(line.to_string());
        }
    }
    None
}

fn extract_serial(output: &str) -> Option<String> {
    for line in output.lines() {
        let line = line.trim();
        if line.contains("Serial Number") || line.contains("Processor board ID") {
            return Some(line.to_string());
        }
    }
    None
}

fn count_interfaces(output: &str) -> (usize, usize) {
    let mut total = 0;
    let mut active = 0;

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || line.contains("Interface") || line.contains("---") {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            total += 1;
            if parts.get(4).unwrap_or(&"").to_lowercase().contains("up") {
                active += 1;
            }
        }
    }

    (total, active)
}
