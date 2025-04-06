use netssh_core::{
    device_connection::{DeviceConfig, NetworkDeviceConnection},
    device_factory::DeviceFactory,
    error::NetsshError,
};
use std::collections::HashMap;
use std::env;
use std::time::Duration;

/// Integration tests for common device functionality
/// These tests require actual device connectivity unless MOCK_TESTS=1 is set
/// Run with: cargo test --test common_tests -- --nocapture
/// Or for mock only: MOCK_TESTS=1 cargo test --test common_tests -- --nocapture

// Helper function to determine if we should run with real devices or mocks
fn use_mock_devices() -> bool {
    match env::var("MOCK_TESTS") {
        Ok(val) => val == "1",
        Err(_) => false,
    }
}

// Get environment variable with fallback
fn get_env_or(name: &str, default: &str) -> String {
    match env::var(name) {
        Ok(val) => val,
        Err(_) => default.to_string(),
    }
}

/// Create a device configuration from environment variables
fn create_device_config(device_type: &str, env_prefix: &str) -> DeviceConfig {
    let host = get_env_or(&format!("{}_HOST", env_prefix), "127.0.0.1");
    let username = get_env_or(&format!("{}_USERNAME", env_prefix), "admin");
    let password = get_env_or(&format!("{}_PASSWORD", env_prefix), "admin");
    let port = get_env_or(&format!("{}_PORT", env_prefix), "22")
        .parse()
        .unwrap_or(22);
    let timeout = get_env_or(&format!("{}_TIMEOUT", env_prefix), "10")
        .parse()
        .unwrap_or(10.0);

    DeviceConfig {
        device_type: device_type.to_string(),
        host,
        username,
        password: Some(password),
        port: Some(port),
        timeout: Some(Duration::from_secs_f64(timeout)),
        secret: None,
        session_log: None,
    }
}

// Mock device implementation
#[cfg(test)]
mod mock {
    use super::*;
    use std::sync::{Arc, Mutex};

    // Import from our utils module
    use crate::utils::mock_device::{MockNetworkDevice, PromptStyle};

    // Generic mock device that can be configured for different device types
    pub struct GenericMockDevice {
        device: Arc<Mutex<MockNetworkDevice>>,
        device_type: String,
    }

    impl GenericMockDevice {
        // Create a new mock device for the specified device type
        pub fn new(device_type: &str, username: &str, password: &str) -> Self {
            let mut device = MockNetworkDevice::new();

            // Basic device configuration
            device
                .set_device_type(device_type)
                .set_hostname("mock-device")
                .add_auth_credentials(username, password);

            // Set appropriate prompt style based on device type
            match device_type {
                "cisco_ios" => {
                    device.set_prompt_style(PromptStyle::CiscoIOS);
                }
                "cisco_nxos" => {
                    device.set_prompt_style(PromptStyle::Custom("mock-device#".to_string()));
                }
                "cisco_asa" => {
                    device.set_prompt_style(PromptStyle::CiscoASA);
                }
                "juniper_junos" => {
                    device.set_prompt_style(PromptStyle::JuniperJunos);
                }
                _ => {
                    device.set_prompt_style(PromptStyle::Custom(format!("{}# ", device_type)));
                }
            }

            // Common command responses for all device types
            device.add_command_response("terminal width 511", "terminal width 511\r\nmock-device#");
            device.add_command_response("terminal length 0", "terminal length 0\r\nmock-device#");
            device.add_command_response("\n", "mock-device#");

            // Version command for all device types
            device.add_command_response("show version", Self::generic_version_output(device_type));

            // Config mode commands for all device types
            device.add_command_response("configure terminal", "Enter configuration commands, one per line. End with CNTL/Z.\r\nmock-device(config)#");
            device.add_command_response("exit", "mock-device#");
            device.add_command_response("end", "mock-device#");

            // Start the mock device
            device.start().expect("Failed to start mock device");

            Self {
                device: Arc::new(Mutex::new(device)),
                device_type: device_type.to_string(),
            }
        }

        // Add additional command responses after creation
        pub fn add_command_response(&self, command: &str, response: &str) {
            if let Ok(mut device) = self.device.lock() {
                device.add_command_response(command, response);
            }
        }

        // Get the port the mock device is listening on
        pub fn port(&self) -> u16 {
            self.device.lock().unwrap().port()
        }

        // Return a generic version output based on device type
        fn generic_version_output(device_type: &str) -> &'static str {
            match device_type {
                "cisco_ios" => {
                    "Cisco IOS Software, C3750E Software (C3750E-UNIVERSALK9-M), Version 15.0(2)SE\r\n\
                     Copyright (c) 1986-2012 by Cisco Systems, Inc.\r\n\
                     \r\n\
                     ROM: Bootstrap program is C3750E boot loader\r\n\
                     BOOTLDR: C3750E Boot Loader (C3750X-HBOOT-M) Version 12.2(58)SE1\r\n\
                     \r\n\
                     mock-device uptime is 10 weeks, 3 days, 5 hours, 1 minute\r\n\
                     System returned to ROM by power-on\r\n\
                     System image file is \"flash:c3750e-universalk9-mz.150-2.SE.bin\"\r\n\
                     \r\n\
                     cisco WS-C3750X-48P (PowerPC405) processor with 262144K bytes of memory.\r\n\
                     Last reset from power-on\r\n\
                     mock-device#"
                },
                "cisco_nxos" => {
                    "Cisco Nexus Operating System (NX-OS) Software\r\n\
                     TAC support: http://www.cisco.com/tac\r\n\
                     Copyright (c) 2002-2019, Cisco Systems, Inc. All rights reserved.\r\n\
                     \r\n\
                     NXOS: version 9.2(3)\r\n\
                     NXOS image file is: bootflash:///nxos.9.2.3.bin\r\n\
                     \r\n\
                     Hardware\r\n\
                       cisco Nexus9000 C9372PX chassis\r\n\
                       Intel(R) Xeon(R) CPU E5-2403 v2 @ 1.80GHz with 16337884 kB of memory.\r\n\
                       Processor Board ID SAL1818S7LV\r\n\
                     \r\n\
                       Device name: mock-device\r\n\
                       bootflash:   51496280 kB\r\n\
                       Kernel uptime is 50 day(s), 23 hour(s), 16 minute(s), 50 second(s)\r\n\
                     \r\n\
                     mock-device#"
                },
                "cisco_asa" => {
                    "Cisco Adaptive Security Appliance Software Version 9.8(2)\r\n\
                     Firepower Extensible Operating System Version 2.2(2.52)\r\n\
                     Device Manager Version 7.8(2)\r\n\
                     \r\n\
                     Compiled on Fri 01-Dec-2017 11:26 PST by builders\r\n\
                     System image file is \"disk0:/asa982-lfbff-k8.SPA\"\r\n\
                     Config file at boot was \"startup-config\"\r\n\
                     \r\n\
                     mock-device up 1 day 2 hours\r\n\
                     \r\n\
                     Hardware:   ASA5525, 8192 MB RAM, CPU Pentium D 2000 MHz,\r\n\
                     Internal ATA Compact Flash, 8GB\r\n\
                     \r\n\
                     mock-device#"
                },
                "juniper_junos" => {
                    "Hostname: mock-device\r\n\
                     Model: mx240\r\n\
                     Junos: 17.3R3-S1.7\r\n\
                     JUNOS OS Kernel 64-bit  [20180704.355248_builder_stable_11]\r\n\
                     JUNOS OS libs [20180704.355248_builder_stable_11]\r\n\
                     JUNOS OS runtime [20180704.355248_builder_stable_11]\r\n\
                     JUNOS OS time zone information [20180704.355248_builder_stable_11]\r\n\
                     JUNOS network stack and utilities [20180714.042739_builder_junos_173_r3_s1]\r\n\
                     JUNOS modules [20180714.042739_builder_junos_173_r3_s1]\r\n\
                     JUNOS mx modules [20180714.042739_builder_junos_173_r3_s1]\r\n\
                     \r\n\
                     user@mock-device>"
                },
                _ => {
                    "Generic device version output for device type\r\n\
                     System version: 1.0\r\n\
                     Hardware: Generic Hardware\r\n\
                     Uptime: 10 days, 5 hours\r\n\
                     mock-device#"
                }
            }
        }
    }

    impl Drop for GenericMockDevice {
        fn drop(&mut self) {
            if let Ok(mut device) = self.device.lock() {
                let _ = device.stop();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock::GenericMockDevice;
    use super::*;

    // Supported device types for testing
    const DEVICE_TYPES: [&str; 4] = ["cisco_ios", "cisco_nxos", "cisco_asa", "juniper_junos"];

    // Test device creation and basic connectivity for all device types
    #[test]
    fn test_create_device() -> Result<(), NetsshError> {
        for device_type in DEVICE_TYPES.iter() {
            println!("Testing device creation for: {}", device_type);

            if use_mock_devices() {
                // Use mock device
                let mock_device = GenericMockDevice::new(device_type, "admin", "admin");
                let port = mock_device.port();

                let mut config = create_device_config(device_type, "DEFAULT");
                config.host = "127.0.0.1".to_string();
                config.port = Some(port);

                let device = DeviceFactory::create_device(&config)?;
                assert_eq!(device.get_device_type(), *device_type);
            } else {
                // Skip real device tests that don't have connection info
                let env_prefix = match *device_type {
                    "cisco_ios" => "IOS",
                    "cisco_nxos" => "NXOS",
                    "cisco_asa" => "ASA",
                    "juniper_junos" => "JUNOS",
                    _ => continue,
                };

                if get_env_or(&format!("{}_HOST", env_prefix), "") == "" {
                    println!(
                        "Skipping real device test for {} (no host specified)",
                        device_type
                    );
                    continue;
                }

                let config = create_device_config(device_type, env_prefix);
                let device = DeviceFactory::create_device(&config)?;
                assert_eq!(device.get_device_type(), *device_type);
            }
        }

        Ok(())
    }

    // Test basic connection for all device types
    #[test]
    fn test_connect() -> Result<(), NetsshError> {
        for device_type in DEVICE_TYPES.iter() {
            println!("Testing connection for: {}", device_type);

            if use_mock_devices() {
                // Use mock device
                let mock_device = GenericMockDevice::new(device_type, "admin", "admin");
                let port = mock_device.port();

                let mut config = create_device_config(device_type, "DEFAULT");
                config.host = "127.0.0.1".to_string();
                config.port = Some(port);

                let mut device = DeviceFactory::create_device(&config)?;
                device.connect()?;
                device.close()?;
            } else {
                // Skip real device tests that don't have connection info
                let env_prefix = match *device_type {
                    "cisco_ios" => "IOS",
                    "cisco_nxos" => "NXOS",
                    "cisco_asa" => "ASA",
                    "juniper_junos" => "JUNOS",
                    _ => continue,
                };

                if get_env_or(&format!("{}_HOST", env_prefix), "") == "" {
                    println!(
                        "Skipping real device test for {} (no host specified)",
                        device_type
                    );
                    continue;
                }

                let config = create_device_config(device_type, env_prefix);
                let mut device = DeviceFactory::create_device(&config)?;
                device.connect()?;
                device.close()?;
            }
        }

        Ok(())
    }

    // Test get_device_info for all device types
    #[test]
    fn test_get_device_info() -> Result<(), NetsshError> {
        for device_type in DEVICE_TYPES.iter() {
            println!("Testing device info for: {}", device_type);

            if use_mock_devices() {
                // Use mock device
                let mock_device = GenericMockDevice::new(device_type, "admin", "admin");
                let port = mock_device.port();

                let mut config = create_device_config(device_type, "DEFAULT");
                config.host = "127.0.0.1".to_string();
                config.port = Some(port);

                let mut device = DeviceFactory::create_device(&config)?;
                device.connect()?;

                let device_info = device.get_device_info()?;
                assert_eq!(device_info.device_type, *device_type);

                // Device type specific assertions
                match *device_type {
                    "cisco_ios" => {
                        assert!(device_info.version.contains("Cisco IOS Software"));
                    }
                    "cisco_nxos" => {
                        assert!(device_info.version.contains("NXOS: version"));
                    }
                    "cisco_asa" => {
                        assert!(device_info
                            .version
                            .contains("Cisco Adaptive Security Appliance"));
                    }
                    "juniper_junos" => {
                        assert!(device_info.version.contains("Junos:"));
                    }
                    _ => {}
                }

                device.close()?;
            } else {
                // Skip real device tests that don't have connection info
                let env_prefix = match *device_type {
                    "cisco_ios" => "IOS",
                    "cisco_nxos" => "NXOS",
                    "cisco_asa" => "ASA",
                    "juniper_junos" => "JUNOS",
                    _ => continue,
                };

                if get_env_or(&format!("{}_HOST", env_prefix), "") == "" {
                    println!(
                        "Skipping real device test for {} (no host specified)",
                        device_type
                    );
                    continue;
                }

                let config = create_device_config(device_type, env_prefix);
                let mut device = DeviceFactory::create_device(&config)?;
                device.connect()?;

                let device_info = device.get_device_info()?;
                assert_eq!(device_info.device_type, *device_type);
                assert!(!device_info.version.is_empty());

                device.close()?;
            }
        }

        Ok(())
    }

    // Test send_command for all device types
    #[test]
    fn test_send_command() -> Result<(), NetsshError> {
        for device_type in DEVICE_TYPES.iter() {
            println!("Testing send_command for: {}", device_type);

            if use_mock_devices() {
                // Use mock device
                let mock_device = GenericMockDevice::new(device_type, "admin", "admin");

                // Add command responses specific to this test
                mock_device.add_command_response(
                    "show clock",
                    "10:30:00.123 UTC Mon Jan 1 2023\r\nmock-device#",
                );
                mock_device.add_command_response(
                    "show interfaces",
                    "Mock interface output\r\nmock-device#",
                );

                let port = mock_device.port();

                let mut config = create_device_config(device_type, "DEFAULT");
                config.host = "127.0.0.1".to_string();
                config.port = Some(port);

                let mut device = DeviceFactory::create_device(&config)?;
                device.connect()?;

                // Test a simple command
                let clock_output = device.send_command("show clock")?;
                assert!(clock_output.contains("10:30:00.123 UTC"));

                // Test a more complex command
                let interfaces_output = device.send_command("show interfaces")?;
                assert!(interfaces_output.contains("Mock interface output"));

                device.close()?;
            } else {
                // Skip real device tests that don't have connection info
                let env_prefix = match *device_type {
                    "cisco_ios" => "IOS",
                    "cisco_nxos" => "NXOS",
                    "cisco_asa" => "ASA",
                    "juniper_junos" => "JUNOS",
                    _ => continue,
                };

                if get_env_or(&format!("{}_HOST", env_prefix), "") == "" {
                    println!(
                        "Skipping real device test for {} (no host specified)",
                        device_type
                    );
                    continue;
                }

                let config = create_device_config(device_type, env_prefix);
                let mut device = DeviceFactory::create_device(&config)?;
                device.connect()?;

                // Test a simple command that should work on all device types
                let version_output = device.send_command("show version")?;
                assert!(!version_output.is_empty());

                device.close()?;
            }
        }

        Ok(())
    }

    // Test config mode for all device types
    #[test]
    fn test_config_mode() -> Result<(), NetsshError> {
        for device_type in DEVICE_TYPES.iter() {
            println!("Testing config mode for: {}", device_type);

            if use_mock_devices() {
                // Use mock device
                let mock_device = GenericMockDevice::new(device_type, "admin", "admin");
                let port = mock_device.port();

                let mut config = create_device_config(device_type, "DEFAULT");
                config.host = "127.0.0.1".to_string();
                config.port = Some(port);

                let mut device = DeviceFactory::create_device(&config)?;
                device.connect()?;

                // Verify not in config mode
                assert!(!device.check_config_mode()?);

                // Enter config mode
                device.enter_config_mode(None)?;

                // Verify in config mode
                assert!(device.check_config_mode()?);

                // Exit config mode
                device.exit_config_mode(None)?;

                // Verify not in config mode anymore
                assert!(!device.check_config_mode()?);

                device.close()?;
            } else {
                // Skip real device tests that don't have connection info
                let env_prefix = match *device_type {
                    "cisco_ios" => "IOS",
                    "cisco_nxos" => "NXOS",
                    "cisco_asa" => "ASA",
                    "juniper_junos" => "JUNOS",
                    _ => continue,
                };

                if get_env_or(&format!("{}_HOST", env_prefix), "") == "" {
                    println!(
                        "Skipping real device test for {} (no host specified)",
                        device_type
                    );
                    continue;
                }

                let config = create_device_config(device_type, env_prefix);
                let mut device = DeviceFactory::create_device(&config)?;
                device.connect()?;

                // Verify not in config mode
                assert!(!device.check_config_mode()?);

                // Enter config mode
                device.enter_config_mode(None)?;

                // Verify in config mode
                assert!(device.check_config_mode()?);

                // Exit config mode
                device.exit_config_mode(None)?;

                // Verify not in config mode anymore
                assert!(!device.check_config_mode()?);

                device.close()?;
            }
        }

        Ok(())
    }

    // Test device-specific paging control
    #[test]
    fn test_disable_paging() -> Result<(), NetsshError> {
        for device_type in DEVICE_TYPES.iter() {
            println!("Testing disable_paging for: {}", device_type);

            if use_mock_devices() {
                // Use mock device
                let mock_device = GenericMockDevice::new(device_type, "admin", "admin");

                // Add responses for paging commands based on device type
                match *device_type {
                    "cisco_ios" | "cisco_nxos" | "cisco_asa" => {
                        mock_device.add_command_response(
                            "terminal length 0",
                            "terminal length 0\r\nmock-device#",
                        );
                    }
                    "juniper_junos" => {
                        mock_device.add_command_response(
                            "set cli screen-length 0",
                            "set cli screen-length 0\r\nmock-device#",
                        );
                    }
                    _ => {}
                }

                let port = mock_device.port();

                let mut config = create_device_config(device_type, "DEFAULT");
                config.host = "127.0.0.1".to_string();
                config.port = Some(port);

                let mut device = DeviceFactory::create_device(&config)?;
                device.connect()?;

                // Test disable_paging
                device.disable_paging()?;

                device.close()?;
            } else {
                // Skip real device tests that don't have connection info
                let env_prefix = match *device_type {
                    "cisco_ios" => "IOS",
                    "cisco_nxos" => "NXOS",
                    "cisco_asa" => "ASA",
                    "juniper_junos" => "JUNOS",
                    _ => continue,
                };

                if get_env_or(&format!("{}_HOST", env_prefix), "") == "" {
                    println!(
                        "Skipping real device test for {} (no host specified)",
                        device_type
                    );
                    continue;
                }

                let config = create_device_config(device_type, env_prefix);
                let mut device = DeviceFactory::create_device(&config)?;
                device.connect()?;

                // Test disable_paging
                device.disable_paging()?;

                device.close()?;
            }
        }

        Ok(())
    }

    // Test device factory with device autodetection
    #[test]
    fn test_device_autodetection() -> Result<(), NetsshError> {
        if use_mock_devices() {
            println!("Testing device autodetection with mock devices");

            // Create mock devices for testing autodetection
            let devices_to_test = DEVICE_TYPES
                .iter()
                .map(|&device_type| {
                    let mock_device = GenericMockDevice::new(device_type, "admin", "admin");
                    (device_type, mock_device)
                })
                .collect::<Vec<_>>();

            for (device_type, mock_device) in devices_to_test {
                let port = mock_device.port();

                let mut config = create_device_config("autodetect", "DEFAULT");
                config.host = "127.0.0.1".to_string();
                config.port = Some(port);

                println!("Testing autodetection for device type: {}", device_type);

                let mut device = DeviceFactory::create_device(&config)?;

                // The device type should match what our mock is pretending to be
                assert_eq!(device.get_device_type(), device_type);

                device.connect()?;
                device.close()?;
            }
        } else {
            // For real devices, we'll just test one device type with autodetection
            println!("Testing device autodetection with real device");

            // Find the first device with a host defined
            for device_type in DEVICE_TYPES.iter() {
                let env_prefix = match *device_type {
                    "cisco_ios" => "IOS",
                    "cisco_nxos" => "NXOS",
                    "cisco_asa" => "ASA",
                    "juniper_junos" => "JUNOS",
                    _ => continue,
                };

                if get_env_or(&format!("{}_HOST", env_prefix), "") == "" {
                    continue; // Skip if no host
                }

                println!(
                    "Testing autodetection for real device type: {}",
                    device_type
                );

                // Create config with "autodetect" device type
                let mut config = create_device_config("autodetect", env_prefix);

                let mut device = DeviceFactory::create_device(&config)?;

                // Check the autodetected type
                println!("Autodetected device type: {}", device.get_device_type());

                device.connect()?;
                device.close()?;

                // Only test one device to save time
                break;
            }
        }

        Ok(())
    }
}
