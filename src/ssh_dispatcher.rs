use crate::error::NetsshError;
use crate::vendors::cisco::{CiscoDeviceConfig, CiscoIosDevice, CiscoXrDevice, CiscoAsaDevice, CiscoBaseConnection};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::time::Duration;

lazy_static! {
    static ref DEVICE_TYPES: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        // Cisco IOS variants
        m.insert("cisco_ios", "cisco_ios");
        m.insert("cisco_xe", "cisco_ios");
        m.insert("cisco_ios_telnet", "cisco_ios");
        
        // Cisco XR variants
        m.insert("cisco_xr", "cisco_xr");
        m.insert("cisco_xr_telnet", "cisco_xr");
        
        // Cisco ASA
        m.insert("cisco_asa", "cisco_asa");
        m.insert("cisco_asa_telnet", "cisco_asa");
        m
    };
}

pub struct ConnectHandler;

impl ConnectHandler {
    pub fn connect(
        device_type: &str,
        host: String,
        username: String,
        password: Option<String>,
        port: Option<u16>,
        timeout: Option<u64>,
        enable_secret: Option<String>,
        session_log: Option<String>,
    ) -> Result<Box<dyn CiscoBaseConnection>, NetsshError> {
        if !DEVICE_TYPES.contains_key(device_type) {
            return Err(NetsshError::UnsupportedDevice(device_type.to_string()));
        }

        let config = CiscoDeviceConfig {
            host,
            username,
            password,
            port,
            timeout: timeout.map(Duration::from_secs),
            secret: enable_secret,
            session_log,
        };

        match device_type {
            "cisco_ios" | "cisco_xe" | "cisco_ios_telnet" => {
                let mut device = CiscoIosDevice::new(config)?;
                device.connect()?;
                Ok(Box::new(device))
            }
            "cisco_xr" | "cisco_xr_telnet" => {
                let mut device = CiscoXrDevice::new(config)?;
                device.connect()?;
                Ok(Box::new(device))
            }
            "cisco_asa" | "cisco_asa_telnet" => {
                let mut device = CiscoAsaDevice::new(config)?;
                device.connect()?;
                Ok(Box::new(device))
            }
            _ => Err(NetsshError::UnsupportedDevice(device_type.to_string())),
        }
    }
}
