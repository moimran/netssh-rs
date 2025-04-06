use netssh_core::{
    device_connection::{DeviceConfig, NetworkDeviceConnection},
    device_factory::DeviceFactory,
    error::NetsshError,
};
use std::env;
use std::time::Duration;

/// Integration tests for Cisco NXOS devices
/// These tests require actual device connectivity unless MOCK_TESTS=1 is set
/// Run with: cargo test --test nxos_tests -- --nocapture
/// Or for mock only: MOCK_TESTS=1 cargo test --test nxos_tests -- --nocapture

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

/// Sets up the device configuration based on environment variables or defaults
fn setup_device_config() -> DeviceConfig {
    let host = get_env_or("NXOS_HOST", "127.0.0.1");
    let username = get_env_or("NXOS_USERNAME", "admin");
    let password = get_env_or("NXOS_PASSWORD", "admin");
    let port = get_env_or("NXOS_PORT", "22").parse().unwrap_or(22);
    let timeout = get_env_or("NXOS_TIMEOUT", "10").parse().unwrap_or(10.0);

    DeviceConfig {
        device_type: "cisco_nxos".to_string(),
        host,
        username,
        password: Some(password),
        port: Some(port),
        timeout: Some(Duration::from_secs_f64(timeout)),
        secret: None,
        session_log: None,
    }
}

// Setup a mock NXOS device for testing
#[cfg(test)]
mod mock {
    use super::*;
    use std::sync::{Arc, Mutex};

    // Import from our utils module
    use crate::utils::mock_device::{MockNetworkDevice, PromptStyle};

    pub struct MockNxosDevice {
        device: Arc<Mutex<MockNetworkDevice>>,
    }

    impl MockNxosDevice {
        pub fn new(username: &str, password: &str) -> Self {
            let mut device = MockNetworkDevice::new();

            // Configure the mock device
            device
                .set_device_type("cisco_nxos")
                .set_hostname("nxos-switch")
                .set_prompt_style(PromptStyle::Custom("nxos-switch#".to_string()))
                .add_auth_credentials(username, password);

            // Add basic command responses
            device.add_command_response("terminal width 511", "terminal width 511\r\nnxos-switch#");
            device.add_command_response("terminal length 0", "terminal length 0\r\nnxos-switch#");
            device.add_command_response(
                "terminal session-timeout 0",
                "terminal session-timeout 0\r\nnxos-switch#",
            );
            device.add_command_response("\n", "nxos-switch#");
            device.add_command_response("show version", MockNxosDevice::show_version_output());
            device.add_command_response(
                "show interface brief",
                MockNxosDevice::show_interface_brief_output(),
            );
            device.add_command_response("show vlan", MockNxosDevice::show_vlan_output());
            device.add_command_response(
                "show running-config",
                MockNxosDevice::show_running_config_output(),
            );

            // Config mode commands
            device.add_command_response("configure terminal", "Enter configuration commands, one per line. End with CNTL/Z.\r\nnxos-switch(config)#");
            device.add_command_response("hostname TEST-NXOS", "nxos-switch(config)#");
            device.add_command_response("interface Ethernet1/1", "nxos-switch(config-if)#");
            device.add_command_response("description Test Interface", "nxos-switch(config-if)#");
            device.add_command_response("no shutdown", "nxos-switch(config-if)#");
            device.add_command_response("exit", "nxos-switch(config)#");
            device.add_command_response("end", "nxos-switch#");
            device.add_command_response(
                "copy running-config startup-config",
                "Copy complete.\r\nnxos-switch#",
            );

            // Start the mock device
            device.start().expect("Failed to start mock device");

            Self {
                device: Arc::new(Mutex::new(device)),
            }
        }

        pub fn port(&self) -> u16 {
            self.device.lock().unwrap().port()
        }

        // Mock command outputs
        fn show_version_output() -> &'static str {
            r#"Cisco Nexus Operating System (NX-OS) Software
TAC support: http://www.cisco.com/tac
Copyright (C) 2002-2020, Cisco Systems, Inc. All rights reserved.
The copyrights to certain works contained in this software are
owned by other third parties and used and distributed under their own
licenses, such as open source.  This software is provided "as is," and unless
otherwise stated, there is no warranty, express or implied, including but not
limited to warranties of merchantability and fitness for a particular purpose.
Certain components of this software are licensed under
the GNU General Public License (GPL) version 2.0 or 
GNU General Public License (GPL) version 3.0  or the GNU
Lesser General Public License (LGPL) Version 2.1 or 
Lesser General Public License (LGPL) Version 2.0. 
A copy of each such license is available at
http://www.opensource.org/licenses/gpl-2.0.php and
http://opensource.org/licenses/gpl-3.0.html and
http://www.opensource.org/licenses/lgpl-2.1.php and
http://www.gnu.org/licenses/old-licenses/library.txt.

NXOS: version 9.3(3)
NXOS image file is: bootflash:///nxos.9.3.3.bin
NXOS compile time: 12/22/2019 2:00:00 [12/22/2019 14:00:00]

Hardware
  cisco Nexus9000 C9336C-FX2 Chassis (Nexus 9000 Series)
  Intel(R) Xeon(R) CPU E5-2403 v2 @ 1.80GHz with 16337884 kB of memory.
  Processor Board ID FDO21120V5D

  Device name: nxos-switch
  bootflash:   51496280 kB
  Kernel uptime is 7 day(s), 14 hour(s), 46 minute(s), 59 second(s)

Last reset 
  Reason: Unknown
  System version: 9.3(3)
  Service: 

plugin
  Core Plugin, Ethernet Plugin

Active Package(s):
nxos-switch# "#
        }

        fn show_interface_brief_output() -> &'static str {
            r#"--------------------------------------------------------------------------------
Port   VRF          Status IP Address                              Speed    MTU
--------------------------------------------------------------------------------
mgmt0  management   up     192.168.1.10                           1000     1500

--------------------------------------------------------------------------------
Ethernet      VLAN    Type Mode   Status  Reason                   Speed     Port
Interface                                                                    Ch #
--------------------------------------------------------------------------------
Eth1/1        1       eth  trunk  up      none                      10G(D)    --
Eth1/2        1       eth  access down    SFP not inserted           auto     --
Eth1/3        1       eth  access down    SFP not inserted           auto     --
Eth1/4        1       eth  trunk  up      none                      10G(D)    --
nxos-switch# "#
        }

        fn show_vlan_output() -> &'static str {
            r#"
VLAN Name                             Status    Ports
---- -------------------------------- --------- -------------------------------
1    default                          active    Eth1/1, Eth1/4
10   DATA                             active    
20   VOICE                            active    
30   MANAGEMENT                       active    

VLAN Type
---- -----
1    enet
10   enet
20   enet
30   enet

Remote SPAN VLANs
----------------

Primary  Secondary  Type             Ports
-------  ---------  ---------------  -------------------------------------------
nxos-switch# "#
        }

        fn show_running_config_output() -> &'static str {
            r#"!Command: show running-config
!Running configuration last done at: Fri Jan 10 11:30:28 2023
!Time: Fri Jan 10 11:35:43 2023

version 9.3(3) Bios:version  
hostname nxos-switch
vdc nxos-switch id 1
  limit-resource vlan minimum 16 maximum 4094
  limit-resource vrf minimum 2 maximum 4096
  limit-resource port-channel minimum 0 maximum 511
  limit-resource u4route-mem minimum 248 maximum 248
  limit-resource u6route-mem minimum 96 maximum 96
  limit-resource m4route-mem minimum 58 maximum 58
  limit-resource m6route-mem minimum 8 maximum 8

feature telnet
feature nxapi
feature bash-shell
feature scp-server
feature interface-vlan

no password strength-check
username admin password 5 $5$KIXGM$O.qF5nPPQ8Fg6h4XDkH/49X3Y9/vL5rAfCNYl4CN83.  role network-admin
username admin passphrase  lifetime 99999 warntime 14 gracetime 3
ip domain-lookup
copp profile strict
snmp-server user admin network-admin auth md5 0x328945d53e05e8e7207f8c20b142f0b7 priv 0x328945d53e05e8e7207f8c20b142f0b7 localizedkey
rmon event 1 description FATAL(1) owner PMON@FATAL
rmon event 2 description CRITICAL(2) owner PMON@CRITICAL
rmon event 3 description ERROR(3) owner PMON@ERROR
rmon event 4 description WARNING(4) owner PMON@WARNING
rmon event 5 description INFORMATION(5) owner PMON@INFO

vlan 1,10,20,30
vlan 10
  name DATA
vlan 20
  name VOICE
vlan 30
  name MANAGEMENT

interface Ethernet1/1
  description Uplink to Core
  no shutdown
  switchport mode trunk

interface Ethernet1/2
  shutdown

interface Ethernet1/3
  shutdown

interface Ethernet1/4
  no shutdown
  switchport mode trunk

interface mgmt0
  vrf member management
  ip address 192.168.1.10/24
nxos-switch# "#
        }
    }

    impl Drop for MockNxosDevice {
        fn drop(&mut self) {
            if let Ok(mut device) = self.device.lock() {
                let _ = device.stop();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock::MockNxosDevice;
    use super::*;

    // Test basic connectivity
    #[test]
    fn test_nxos_connect() -> Result<(), NetsshError> {
        if use_mock_devices() {
            // Use mock device
            let mock_device = MockNxosDevice::new("admin", "admin");
            let port = mock_device.port();

            let mut config = setup_device_config();
            config.host = "127.0.0.1".to_string();
            config.port = Some(port);

            let mut device = DeviceFactory::create_device(&config)?;
            device.connect()?;
            device.close()?;
            Ok(())
        } else {
            // Use real device
            let config = setup_device_config();
            let mut device = DeviceFactory::create_device(&config)?;
            device.connect()?;
            device.close()?;
            Ok(())
        }
    }

    // Test command execution
    #[test]
    fn test_nxos_send_command() -> Result<(), NetsshError> {
        if use_mock_devices() {
            // Use mock device
            let mock_device = MockNxosDevice::new("admin", "admin");
            let port = mock_device.port();

            let mut config = setup_device_config();
            config.host = "127.0.0.1".to_string();
            config.port = Some(port);

            let mut device = DeviceFactory::create_device(&config)?;
            device.connect()?;

            // Execute commands
            let version_output = device.send_command("show version")?;
            assert!(version_output.contains("NXOS: version"));
            assert!(version_output.contains("cisco Nexus"));

            device.close()?;
            Ok(())
        } else {
            // Use real device
            let config = setup_device_config();
            let mut device = DeviceFactory::create_device(&config)?;
            device.connect()?;

            // Execute commands
            let version_output = device.send_command("show version")?;
            assert!(version_output.contains("NXOS: version"));

            device.close()?;
            Ok(())
        }
    }

    // Test configuration commands
    #[test]
    fn test_nxos_config_mode() -> Result<(), NetsshError> {
        if use_mock_devices() {
            // Use mock device
            let mock_device = MockNxosDevice::new("admin", "admin");
            let port = mock_device.port();

            let mut config = setup_device_config();
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
            Ok(())
        } else {
            // Use real device
            let config = setup_device_config();
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
            Ok(())
        }
    }

    // Test configuration commands in sequence
    #[test]
    fn test_nxos_send_config_commands() -> Result<(), NetsshError> {
        if use_mock_devices() {
            // Use mock device
            let mock_device = MockNxosDevice::new("admin", "admin");
            let port = mock_device.port();

            let mut config = setup_device_config();
            config.host = "127.0.0.1".to_string();
            config.port = Some(port);

            let mut device = DeviceFactory::create_device(&config)?;
            device.connect()?;

            // Send configuration commands
            let commands = vec![
                "interface Ethernet1/1",
                "description Test Interface",
                "no shutdown",
                "exit",
            ];

            let results = device.send_config_commands(&commands)?;

            // Verify command results
            assert_eq!(results.len(), commands.len());

            // Verify interface configuration
            let interface_output =
                device.send_command("show running-config interface Ethernet1/1")?;
            assert!(interface_output.contains("description Test Interface"));

            device.close()?;
            Ok(())
        } else {
            // Use real device
            let config = setup_device_config();
            let mut device = DeviceFactory::create_device(&config)?;
            device.connect()?;

            // Send configuration commands - using dummy commands that are safe to run
            let commands = vec![
                "interface Ethernet1/1",
                "description Test Interface from netssh-rs",
                "exit",
            ];

            let results = device.send_config_commands(&commands)?;

            // Verify command results
            assert_eq!(results.len(), commands.len());

            // Verify interface configuration
            let interface_output =
                device.send_command("show running-config interface Ethernet1/1")?;
            assert!(interface_output.contains("description Test Interface from netssh-rs"));

            // Revert changes
            let revert_commands = vec!["interface Ethernet1/1", "no description", "exit"];

            device.send_config_commands(&revert_commands)?;

            device.close()?;
            Ok(())
        }
    }

    // Test get_device_info
    #[test]
    fn test_nxos_device_info() -> Result<(), NetsshError> {
        if use_mock_devices() {
            // Use mock device
            let mock_device = MockNxosDevice::new("admin", "admin");
            let port = mock_device.port();

            let mut config = setup_device_config();
            config.host = "127.0.0.1".to_string();
            config.port = Some(port);

            let mut device = DeviceFactory::create_device(&config)?;
            device.connect()?;

            // Get device info
            let device_info = device.get_device_info()?;

            // Verify info
            assert_eq!(device_info.device_type, "cisco_nxos");
            assert!(device_info.version.contains("NXOS: version"));
            assert!(device_info.model.contains("cisco Nexus"));

            device.close()?;
            Ok(())
        } else {
            // Use real device
            let config = setup_device_config();
            let mut device = DeviceFactory::create_device(&config)?;
            device.connect()?;

            // Get device info
            let device_info = device.get_device_info()?;

            // Verify info
            assert_eq!(device_info.device_type, "cisco_nxos");
            assert!(!device_info.version.is_empty());

            device.close()?;
            Ok(())
        }
    }

    // Test save configuration (careful with real devices!)
    #[test]
    fn test_nxos_save_configuration() -> Result<(), NetsshError> {
        if use_mock_devices() {
            // Use mock device
            let mock_device = MockNxosDevice::new("admin", "admin");
            let port = mock_device.port();

            let mut config = setup_device_config();
            config.host = "127.0.0.1".to_string();
            config.port = Some(port);

            let mut device = DeviceFactory::create_device(&config)?;
            device.connect()?;

            // Save configuration
            device.save_configuration()?;

            device.close()?;
            Ok(())
        } else {
            // This test is commented out for real devices to avoid saving configuration changes
            // Uncomment if you want to test save_configuration on a real device
            /*
            let config = setup_device_config();
            let mut device = DeviceFactory::create_device(&config)?;
            device.connect()?;

            // Save configuration
            device.save_configuration()?;

            device.close()?;
            */
            Ok(())
        }
    }
}
