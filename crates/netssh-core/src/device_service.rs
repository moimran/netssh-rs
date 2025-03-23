use crate::device_connection::{DeviceInfo, NetworkDeviceConnection};
use crate::error::NetsshError;
use log::{info};

/// Basic interface information
#[derive(Debug, Clone)]
pub struct Interface {
    pub name: String,
    pub status: String,
    pub ip_address: Option<String>,
    pub description: Option<String>,
}

/// Service layer for network device operations
pub struct DeviceService<T: NetworkDeviceConnection> {
    device: T,
}

impl<T: NetworkDeviceConnection> DeviceService<T> {
    /// Create a new device service with the given device connection
    pub fn new(device: T) -> Self {
        Self { device }
    }
    
    /// Connect to the device
    pub fn connect(&mut self) -> Result<(), NetsshError> {
        info!("Connecting to device");
        self.device.connect()
    }
    
    /// Close the connection to the device
    pub fn close(&mut self) -> Result<(), NetsshError> {
        info!("Closing connection to device");
        self.device.close()
    }
    
    /// Get device information
    pub fn get_device_info(&mut self) -> Result<DeviceInfo, NetsshError> {
        info!("Getting device information");
        let device_type = self.device.get_device_type().to_string();
        
        // Use device_type to determine the appropriate command
        let command = match device_type.as_str() {
            t if t.contains("cisco") => "show version",
            t if t.contains("juniper") => "show version",
            _ => return Err(NetsshError::UnsupportedOperation(
                format!("Getting device info not supported for {}", device_type)
            )),
        };
        
        let output = self.device.send_command(command)?;
        
        // Parse the output based on device_type
        let info = match device_type.as_str() {
            t if t.contains("cisco_ios") => parse_cisco_ios_version(&output),
            t if t.contains("cisco_xr") => parse_cisco_xr_version(&output),
            t if t.contains("cisco_nxos") => parse_cisco_nxos_version(&output),
            t if t.contains("cisco_asa") => parse_cisco_asa_version(&output),
            t if t.contains("juniper") => parse_juniper_version(&output),
            _ => DeviceInfo {
                vendor: device_type.to_string(),
                model: "Unknown".to_string(),
                os_version: "Unknown".to_string(),
                hostname: "Unknown".to_string(),
                uptime: "Unknown".to_string(),
            },
        };
        
        Ok(info)
    }
    
    /// Get interfaces from the device
    pub fn get_interfaces(&mut self) -> Result<Vec<Interface>, NetsshError> {
        info!("Getting interfaces from device");
        let device_type = self.device.get_device_type().to_string();
        
        // Use device_type to determine the appropriate command
        let command = match device_type.as_str() {
            t if t.contains("cisco_ios") => "show interfaces status",
            t if t.contains("cisco_xr") => "show interfaces brief",
            t if t.contains("cisco_nxos") => "show interface status",
            t if t.contains("cisco_asa") => "show interface",
            t if t.contains("juniper") => "show interfaces terse",
            _ => return Err(NetsshError::UnsupportedOperation(
                format!("Getting interfaces not supported for {}", device_type)
            )),
        };
        
        let output = self.device.send_command(command)?;
        
        // Parse the output based on device_type
        let interfaces = match device_type.as_str() {
            t if t.contains("cisco_ios") => parse_cisco_ios_interfaces(&output),
            t if t.contains("cisco_xr") => parse_cisco_xr_interfaces(&output),
            t if t.contains("cisco_nxos") => parse_cisco_nxos_interfaces(&output),
            t if t.contains("cisco_asa") => parse_cisco_asa_interfaces(&output),
            t if t.contains("juniper") => parse_juniper_interfaces(&output),
            _ => vec![],
        };
        
        Ok(interfaces)
    }
    
    /// Configure an interface
    pub fn configure_interface(&mut self, interface_name: &str, description: &str) -> Result<(), NetsshError> {
        info!("Configuring interface {} with description: {}", interface_name, description);
        let device_type = self.device.get_device_type().to_string();
        
        // Enter config mode
        self.device.enter_config_mode(None)?;
        
        // Configure interface based on device type
        match device_type.as_str() {
            t if t.contains("cisco") => {
                self.device.send_command(&format!("interface {}", interface_name))?;
                self.device.send_command(&format!("description {}", description))?;
            },
            t if t.contains("juniper") => {
                self.device.send_command(&format!("set interfaces {} description \"{}\"", 
                                                 interface_name, description))?;
            },
            _ => {
                // Exit config mode before returning error
                let _ = self.device.exit_config_mode(None);
                return Err(NetsshError::UnsupportedOperation(
                    format!("Configuring interfaces not supported for {}", device_type)
                ));
            },
        }
        
        // Save configuration
        self.device.save_configuration()?;
        
        // Exit config mode
        self.device.exit_config_mode(None)?;
        
        Ok(())
    }
    
    /// Execute a command on the device
    pub fn execute_command(&mut self, command: &str) -> Result<String, NetsshError> {
        info!("Executing command: {}", command);
        self.device.send_command(command)
    }
}

// Helper functions for parsing device output

fn parse_cisco_ios_version(output: &str) -> DeviceInfo {
    // Simple parsing logic - in a real implementation, this would be more robust
    let mut info = DeviceInfo {
        vendor: "Cisco".to_string(),
        model: "Unknown".to_string(),
        os_version: "Unknown".to_string(),
        hostname: "Unknown".to_string(),
        uptime: "Unknown".to_string(),
    };
    
    for line in output.lines() {
        if line.contains("IOS Software") {
            info.os_version = line.trim().to_string();
        } else if line.contains("uptime is") {
            info.uptime = line.trim().to_string();
        } else if line.contains("processor") && line.contains("with") {
            info.model = line.trim().to_string();
        }
    }
    
    info
}

fn parse_cisco_xr_version(output: &str) -> DeviceInfo {
    // Simple parsing logic - in a real implementation, this would be more robust
    let mut info = DeviceInfo {
        vendor: "Cisco".to_string(),
        model: "Unknown".to_string(),
        os_version: "Unknown".to_string(),
        hostname: "Unknown".to_string(),
        uptime: "Unknown".to_string(),
    };
    
    for line in output.lines() {
        if line.contains("Cisco IOS XR Software") {
            info.os_version = line.trim().to_string();
        } else if line.contains("uptime is") {
            info.uptime = line.trim().to_string();
        } else if line.contains("processor") && line.contains("with") {
            info.model = line.trim().to_string();
        }
    }
    
    info
}

fn parse_cisco_nxos_version(output: &str) -> DeviceInfo {
    // Simple parsing logic - in a real implementation, this would be more robust
    let mut info = DeviceInfo {
        vendor: "Cisco".to_string(),
        model: "Unknown".to_string(),
        os_version: "Unknown".to_string(),
        hostname: "Unknown".to_string(),
        uptime: "Unknown".to_string(),
    };
    
    for line in output.lines() {
        if line.contains("NXOS:") {
            info.os_version = line.trim().to_string();
        } else if line.contains("uptime is") {
            info.uptime = line.trim().to_string();
        } else if line.contains("Hardware") {
            info.model = line.trim().to_string();
        }
    }
    
    info
}

fn parse_cisco_asa_version(output: &str) -> DeviceInfo {
    // Simple parsing logic - in a real implementation, this would be more robust
    let mut info = DeviceInfo {
        vendor: "Cisco".to_string(),
        model: "ASA".to_string(),
        os_version: "Unknown".to_string(),
        hostname: "Unknown".to_string(),
        uptime: "Unknown".to_string(),
    };
    
    for line in output.lines() {
        if line.contains("Cisco Adaptive Security Appliance Software Version") {
            info.os_version = line.trim().to_string();
        } else if line.contains("up") && line.contains("days") {
            info.uptime = line.trim().to_string();
        } else if line.contains("Hardware:") {
            info.model = line.trim().to_string();
        }
    }
    
    info
}

fn parse_juniper_version(output: &str) -> DeviceInfo {
    // Simple parsing logic - in a real implementation, this would be more robust
    let mut info = DeviceInfo {
        vendor: "Juniper".to_string(),
        model: "Unknown".to_string(),
        os_version: "Unknown".to_string(),
        hostname: "Unknown".to_string(),
        uptime: "Unknown".to_string(),
    };
    
    for line in output.lines() {
        if line.contains("Junos:") {
            info.os_version = line.trim().to_string();
        } else if line.contains("uptime:") {
            info.uptime = line.trim().to_string();
        } else if line.contains("Model:") {
            info.model = line.trim().to_string();
        }
    }
    
    info
}

fn parse_cisco_ios_interfaces(output: &str) -> Vec<Interface> {
    let mut interfaces = Vec::new();
    let mut lines = output.lines();
    
    // Skip header lines
    for _ in 0..2 {
        if lines.next().is_none() {
            return interfaces;
        }
    }
    
    for line in lines {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            interfaces.push(Interface {
                name: parts[0].to_string(),
                status: parts[1].to_string(),
                ip_address: None,
                description: if parts.len() > 3 {
                    Some(parts[3..].join(" "))
                } else {
                    None
                },
            });
        }
    }
    
    interfaces
}

fn parse_cisco_xr_interfaces(output: &str) -> Vec<Interface> {
    let mut interfaces = Vec::new();
    let mut lines = output.lines();
    
    // Skip header lines
    for _ in 0..2 {
        if lines.next().is_none() {
            return interfaces;
        }
    }
    
    for line in lines {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            interfaces.push(Interface {
                name: parts[0].to_string(),
                status: parts[1].to_string(),
                ip_address: if parts.len() > 2 && parts[2] != "unassigned" {
                    Some(parts[2].to_string())
                } else {
                    None
                },
                description: None,
            });
        }
    }
    
    interfaces
}

fn parse_cisco_nxos_interfaces(output: &str) -> Vec<Interface> {
    let mut interfaces = Vec::new();
    let mut lines = output.lines();
    
    // Skip header lines
    for _ in 0..2 {
        if lines.next().is_none() {
            return interfaces;
        }
    }
    
    for line in lines {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            interfaces.push(Interface {
                name: parts[0].to_string(),
                status: parts[2].to_string(),
                ip_address: None,
                description: if parts.len() > 3 {
                    Some(parts[3..].join(" "))
                } else {
                    None
                },
            });
        }
    }
    
    interfaces
}

fn parse_cisco_asa_interfaces(output: &str) -> Vec<Interface> {
    let mut interfaces = Vec::new();
    let mut current_interface: Option<Interface> = None;
    
    for line in output.lines() {
        if line.starts_with("Interface") {
            // Save previous interface if exists
            if let Some(interface) = current_interface.take() {
                interfaces.push(interface);
            }
            
            // Start new interface
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                current_interface = Some(Interface {
                    name: parts[1].trim_matches(',').to_string(),
                    status: "Unknown".to_string(),
                    ip_address: None,
                    description: None,
                });
            }
        } else if let Some(ref mut interface) = current_interface {
            if line.contains("line protocol is") {
                let parts: Vec<&str> = line.split("line protocol is").collect();
                if parts.len() >= 2 {
                    interface.status = parts[1].trim().to_string();
                }
            } else if line.contains("IP address") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    interface.ip_address = Some(parts[2].to_string());
                }
            } else if line.contains("Description:") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 2 {
                    interface.description = Some(parts[1].trim().to_string());
                }
            }
        }
    }
    
    // Add the last interface if exists
    if let Some(interface) = current_interface {
        interfaces.push(interface);
    }
    
    interfaces
}

fn parse_juniper_interfaces(output: &str) -> Vec<Interface> {
    let mut interfaces = Vec::new();
    let mut lines = output.lines();
    
    // Skip header line
    if lines.next().is_none() {
        return interfaces;
    }
    
    for line in lines {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            interfaces.push(Interface {
                name: parts[0].to_string(),
                status: parts[1].to_string(),
                ip_address: if parts.len() > 2 && parts[2] != "--" {
                    Some(parts[2].to_string())
                } else {
                    None
                },
                description: None,
            });
        }
    }
    
    interfaces
}
