use netssh_core::{
    device_connection::{DeviceConfig, NetworkDeviceConnection},
    device_factory::DeviceFactory,
    error::NetsshError,
};
use std::env;
use std::time::Duration;

/// Integration tests for Cisco NXOS-specific features
/// These tests require actual device connectivity unless MOCK_TESTS=1 is set
/// Run with: cargo test --test nxos_specific_tests -- --nocapture
/// Or for mock only: MOCK_TESTS=1 cargo test --test nxos_specific_tests -- --nocapture

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

// Setup a mock NXOS device for testing NXOS-specific features
#[cfg(test)]
mod mock {
    use super::*;
    use std::sync::{Arc, Mutex};

    // Import from our utils module
    use crate::utils::mock_device::{MockNetworkDevice, PromptStyle};

    pub struct NxosSpecificMockDevice {
        device: Arc<Mutex<MockNetworkDevice>>,
    }

    impl NxosSpecificMockDevice {
        pub fn new(username: &str, password: &str) -> Self {
            let mut device = MockNetworkDevice::new();

            // Configure the mock device
            device
                .set_device_type("cisco_nxos")
                .set_hostname("nxos-specific")
                .set_prompt_style(PromptStyle::Custom("nxos-specific#".to_string()))
                .add_auth_credentials(username, password);

            // Basic commands
            device
                .add_command_response("terminal width 511", "terminal width 511\r\nnxos-specific#");
            device.add_command_response("terminal length 0", "terminal length 0\r\nnxos-specific#");
            device.add_command_response(
                "terminal session-timeout 0",
                "terminal session-timeout 0\r\nnxos-specific#",
            );
            device.add_command_response("\n", "nxos-specific#");

            // NXOS-specific command responses
            device.add_command_response(
                "show interface status",
                Self::show_interface_status_output(),
            );
            device.add_command_response("show version", Self::show_version_output());
            device.add_command_response("show feature", Self::show_feature_output());
            device.add_command_response("show vpc", Self::show_vpc_output());
            device.add_command_response("show vrf", Self::show_vrf_output());
            device.add_command_response("show vlan", Self::show_vlan_output());
            device.add_command_response("show fex", Self::show_fex_output());
            device
                .add_command_response("show interface brief", Self::show_interface_brief_output());

            // Config mode commands
            device.add_command_response("configure terminal", "Enter configuration commands, one per line. End with CNTL/Z.\r\nnxos-specific(config)#");
            device.add_command_response("feature nxapi", "nxos-specific(config)#");
            device.add_command_response("feature lldp", "nxos-specific(config)#");
            device.add_command_response("feature vpc", "nxos-specific(config)#");
            device.add_command_response("feature interface-vlan", "nxos-specific(config)#");
            device.add_command_response("vlan 100", "nxos-specific(config-vlan)#");
            device.add_command_response("name TEST_VLAN", "nxos-specific(config-vlan)#");
            device.add_command_response("exit", "nxos-specific(config)#");
            device.add_command_response("interface Ethernet1/1", "nxos-specific(config-if)#");
            device.add_command_response("description Test Interface", "nxos-specific(config-if)#");
            device.add_command_response("switchport", "nxos-specific(config-if)#");
            device.add_command_response("switchport mode access", "nxos-specific(config-if)#");
            device.add_command_response("switchport access vlan 100", "nxos-specific(config-if)#");
            device.add_command_response("no shutdown", "nxos-specific(config-if)#");
            device.add_command_response("end", "nxos-specific#");
            device.add_command_response(
                "copy running-config startup-config",
                "Copy complete.\r\nnxos-specific#",
            );

            // Configuration validation commands
            device.add_command_response(
                "show running-config interface Ethernet1/1",
                Self::show_running_config_interface_output(),
            );
            device.add_command_response(
                "show running-config vlan 100",
                Self::show_running_config_vlan_output(),
            );
            device.add_command_response(
                "show running-config | include feature",
                Self::show_running_config_feature_output(),
            );

            // Start the mock device
            device
                .start()
                .expect("Failed to start NXOS-specific mock device");

            Self {
                device: Arc::new(Mutex::new(device)),
            }
        }

        pub fn port(&self) -> u16 {
            self.device.lock().unwrap().port()
        }

        // Mock command outputs for NXOS-specific features
        fn show_version_output() -> &'static str {
            r#"Cisco Nexus Operating System (NX-OS) Software
TAC support: http://www.cisco.com/tac
Copyright (C) 2002-2022, Cisco Systems, Inc. All rights reserved.
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

NXOS: version 9.3(10)
BIOS: version 07.67
NXOS image file is: bootflash:///nxos.9.3.10.bin
NXOS compile time: 5/17/2022 23:00:00 [05/18/2022 07:00:00]

Hardware
  cisco Nexus9000 C93180YC-FX Chassis
  Intel(R) Xeon(R) CPU D-1526 @ 1.80GHz with 16337884 kB of memory.
  Processor Board ID FDO232112AB

  Device name: nxos-specific
  bootflash:   51496280 kB
  Kernel uptime is 125 day(s), 10 hour(s), 17 minute(s), 53 second(s)

Last reset
  Reason: Unknown
  System version: 9.3(10)
  Service:

plugin
  Core Plugin, Ethernet Plugin

Active Package(s):
nxos-specific# "#
        }

        fn show_interface_status_output() -> &'static str {
            r#"--------------------------------------------------------------------------------
Port          Name               Status    Vlan      Duplex  Speed   Type
--------------------------------------------------------------------------------
Eth1/1        Server-1           connected 100       full    1000    10G
Eth1/2        Server-2           connected 100       full    1000    10G
Eth1/3        TOR-Switch         connected trunk     full    10G     10G
Eth1/4                           notconnect 1        auto    auto    10G
Eth1/5                           notconnect 1        auto    auto    10G
Eth1/6                           notconnect 1        auto    auto    10G
Eth1/7                           notconnect 1        auto    auto    10G
Eth1/8                           notconnect 1        auto    auto    10G
Eth1/9                           notconnect 1        auto    auto    10G
Eth1/10                          notconnect 1        auto    auto    10G
nxos-specific# "#
        }

        fn show_feature_output() -> &'static str {
            r#"Feature Name          Instance  State
------------------------------ -------- --------
assoc_mgr                 1        enabled
bfd                       1        disabled
bgp                       1        disabled
dhcp                      1        disabled
eigrp                     1        disabled
eigrp                     2        disabled
eigrp                     3        disabled
eigrp                     4        disabled
evb                       1        disabled
fabric_ext                1        disabled
fcoe                      1        disabled
glbp                      1        disabled
hsrp_engine               1        disabled
interface-vlan            1        enabled
lacp                      1        enabled
ldap                      1        disabled
lldp                      1        enabled
msdp                      1        disabled
nat                       1        disabled
netflow                   1        disabled
ntp                       1        enabled
nv overlay                1        disabled
nxapi                     1        enabled
ospf                      1        disabled
ospf                      2        disabled
ospf                      3        disabled
ospf                      4        disabled
ospfv3                    1        disabled
ospfv3                    2        disabled
ospfv3                    3        disabled
ospfv3                    4        disabled
otv                       1        disabled
pim                       1        disabled
pim6                      1        disabled
ptp                       1        disabled
rip                       1        disabled
rip                       2        disabled
rip                       3        disabled
rip                       4        disabled
sflow                     1        disabled
ssh                       1        enabled
tacacs                    1        enabled
telnet                    1        enabled
udld                      1        disabled
vpc                       1        enabled
vrrp                      1        disabled
vtp                       1        disabled
nxos-specific# "#
        }

        fn show_vpc_output() -> &'static str {
            r#"Legend:
                (*) - local vPC is down, forwarding via vPC peer-link

vPC domain id                     : 10
Peer status                       : peer adjacency formed ok
vPC keep-alive status             : peer is alive
Configuration consistency status  : success
Per-vlan consistency status       : success
Type-2 consistency status         : success
vPC role                          : primary
Number of vPCs configured         : 2
Peer Gateway                      : Disabled
Dual-active excluded VLANs        : -
Graceful Consistency Check        : Enabled
Auto-recovery status              : Disabled
Delay-restore status              : Timer is off.(timeout = 30s)
Delay-restore SVI status          : Timer is off.(timeout = 10s)
Operational Layer3 Peer-router    : Disabled
Virtual-peerlink mode             : Disabled

vPC Peer-link status
---------------------------------------------------------------------
id    Port   Status Active vlans
--    ----   ------ -------------------------------------------------
1     Po10   up     1,100-105

vPC status
----------------------------------------------------------------------------
Id    Port          Status Consistency Reason                Active vlans
--    ------------  ------ ----------- ------                ---------------
20    Po20          up     success     success               100-105
30    Po30          up     success     success               100-105
nxos-specific# "#
        }

        fn show_vrf_output() -> &'static str {
            r#"VRF-Name                           VRF-ID State   Reason
management                         1      Up      --
prod                               2      Up      --
test                               3      Up      --
nxos-specific# "#
        }

        fn show_vlan_output() -> &'static str {
            r#"
VLAN Name                             Status    Ports
---- -------------------------------- --------- -------------------------------
1    default                          active    Eth1/4, Eth1/5, Eth1/6, Eth1/7
                                                Eth1/8, Eth1/9, Eth1/10
100  PROD_DATA                        active    Eth1/1, Eth1/2
101  PROD_VOICE                       active    
102  PROD_MGMT                        active    
103  TEST_DATA                        active    
104  TEST_VOICE                       active    
105  TEST_MGMT                        active    

VLAN Type
---- -----
1    enet
100  enet
101  enet
102  enet
103  enet
104  enet
105  enet

Remote SPAN VLANs
----------------

Primary  Secondary  Type             Ports
-------  ---------  ---------------  -------------------------------------------
nxos-specific# "#
        }

        fn show_fex_output() -> &'static str {
            r#"FEX         FEX           FEX                       FEX
Number    Description  State      Model            Serial
------------------------------------------------------------------------
100       FEX-100      Online     N2K-C2248TP-E-1G SSI1912387V
101       FEX-101      Online     N2K-C2248TP-E-1G SSI1912388K
nxos-specific# "#
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
Eth1/1        100     eth  access up      none                      1000(D)   --
Eth1/2        100     eth  access up      none                      1000(D)   --
Eth1/3        1       eth  trunk  up      none                      10G(D)    --
Eth1/4        1       eth  access down    SFP not inserted           auto     --
Eth1/5        1       eth  access down    SFP not inserted           auto     --
Eth1/6        1       eth  access down    SFP not inserted           auto     --
Eth1/7        1       eth  access down    SFP not inserted           auto     --
Eth1/8        1       eth  access down    SFP not inserted           auto     --
Eth1/9        1       eth  access down    SFP not inserted           auto     --
Eth1/10       1       eth  access down    SFP not inserted           auto     --

--------------------------------------------------------------------------------
Port-channel VLAN    Type Mode   Status  Reason                    Speed   Protocol
Interface                                                                  
--------------------------------------------------------------------------------
Po10         1       eth  trunk  up      none                       a-10G(D)  lacp
Po20         100-105 eth  trunk  up      none                       a-10G(D)  lacp
Po30         100-105 eth  trunk  up      none                       a-10G(D)  lacp
nxos-specific# "#
        }

        fn show_running_config_interface_output() -> &'static str {
            r#"!Command: show running-config interface Ethernet1/1
!Running configuration last done at: Thu Aug 10 11:45:38 2023
!Time: Thu Aug 10 11:48:23 2023

version 9.3(10)

interface Ethernet1/1
  description Test Interface
  switchport
  switchport access vlan 100
  no shutdown
nxos-specific# "#
        }

        fn show_running_config_vlan_output() -> &'static str {
            r#"!Command: show running-config vlan 100
!Running configuration last done at: Thu Aug 10 11:45:38 2023
!Time: Thu Aug 10 11:48:32 2023

version 9.3(10)

vlan 100
  name TEST_VLAN
nxos-specific# "#
        }

        fn show_running_config_feature_output() -> &'static str {
            r#"feature nxapi
feature lldp
feature vpc
feature interface-vlan
nxos-specific# "#
        }
    }

    impl Drop for NxosSpecificMockDevice {
        fn drop(&mut self) {
            if let Ok(mut device) = self.device.lock() {
                let _ = device.stop();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock::NxosSpecificMockDevice;
    use super::*;

    // Test NXOS-specific device features
    #[test]
    fn test_nxos_feature_management() -> Result<(), NetsshError> {
        if use_mock_devices() {
            // Use mock device
            let mock_device = NxosSpecificMockDevice::new("admin", "admin");
            let port = mock_device.port();

            let mut config = setup_device_config();
            config.host = "127.0.0.1".to_string();
            config.port = Some(port);

            let mut device = DeviceFactory::create_device(&config)?;
            device.connect()?;

            // Check enabled features
            let features_output = device.send_command("show feature")?;

            // Verify that certain features are enabled
            assert!(features_output.contains("nxapi                     1        enabled"));
            assert!(features_output.contains("lldp                      1        enabled"));
            assert!(features_output.contains("vpc                       1        enabled"));

            // Configure a new feature
            let config_commands = vec!["feature interface-vlan"];

            device.send_config_commands(&config_commands)?;

            // Verify feature is enabled
            let updated_features = device.send_command("show running-config | include feature")?;
            assert!(updated_features.contains("feature interface-vlan"));

            device.close()?;
            Ok(())
        } else {
            // Use real device
            let config = setup_device_config();
            let mut device = DeviceFactory::create_device(&config)?;
            device.connect()?;

            // Check enabled features
            let features_output = device.send_command("show feature")?;
            println!("Current enabled features:\n{}", features_output);

            // Only check features, don't modify on real device

            device.close()?;
            Ok(())
        }
    }

    // Test NXOS-specific VPC feature
    #[test]
    fn test_nxos_vpc_status() -> Result<(), NetsshError> {
        if use_mock_devices() {
            // Use mock device
            let mock_device = NxosSpecificMockDevice::new("admin", "admin");
            let port = mock_device.port();

            let mut config = setup_device_config();
            config.host = "127.0.0.1".to_string();
            config.port = Some(port);

            let mut device = DeviceFactory::create_device(&config)?;
            device.connect()?;

            // Check VPC status
            let vpc_output = device.send_command("show vpc")?;

            // Verify VPC information
            assert!(vpc_output.contains("vPC domain id"));
            assert!(vpc_output.contains("Peer status"));
            assert!(vpc_output.contains("vPC Peer-link status"));

            device.close()?;
            Ok(())
        } else {
            // Use real device
            let config = setup_device_config();
            let mut device = DeviceFactory::create_device(&config)?;
            device.connect()?;

            // Try to check VPC status (might not be configured on all devices)
            let vpc_output = device.send_command("show vpc")?;
            println!("VPC Status (if configured):\n{}", vpc_output);

            device.close()?;
            Ok(())
        }
    }

    // Test NXOS-specific VLAN configuration
    #[test]
    fn test_nxos_vlan_configuration() -> Result<(), NetsshError> {
        if use_mock_devices() {
            // Use mock device
            let mock_device = NxosSpecificMockDevice::new("admin", "admin");
            let port = mock_device.port();

            let mut config = setup_device_config();
            config.host = "127.0.0.1".to_string();
            config.port = Some(port);

            let mut device = DeviceFactory::create_device(&config)?;
            device.connect()?;

            // Check current VLANs
            let initial_vlans = device.send_command("show vlan")?;

            // Configure a new VLAN
            let config_commands = vec!["vlan 100", "name TEST_VLAN", "exit"];

            device.send_config_commands(&config_commands)?;

            // Verify VLAN configuration
            let vlan_config = device.send_command("show running-config vlan 100")?;
            assert!(vlan_config.contains("vlan 100"));
            assert!(vlan_config.contains("name TEST_VLAN"));

            device.close()?;
            Ok(())
        } else {
            // Use real device
            let config = setup_device_config();
            let mut device = DeviceFactory::create_device(&config)?;
            device.connect()?;

            // Check current VLANs
            let vlans = device.send_command("show vlan")?;
            println!("Current VLANs:\n{}", vlans);

            // Don't create VLANs on real device

            device.close()?;
            Ok(())
        }
    }

    // Test NXOS-specific VRF feature
    #[test]
    fn test_nxos_vrf_status() -> Result<(), NetsshError> {
        if use_mock_devices() {
            // Use mock device
            let mock_device = NxosSpecificMockDevice::new("admin", "admin");
            let port = mock_device.port();

            let mut config = setup_device_config();
            config.host = "127.0.0.1".to_string();
            config.port = Some(port);

            let mut device = DeviceFactory::create_device(&config)?;
            device.connect()?;

            // Check VRF status
            let vrf_output = device.send_command("show vrf")?;

            // Verify VRF information
            assert!(vrf_output.contains("management"));
            assert!(vrf_output.contains("prod"));
            assert!(vrf_output.contains("test"));

            device.close()?;
            Ok(())
        } else {
            // Use real device
            let config = setup_device_config();
            let mut device = DeviceFactory::create_device(&config)?;
            device.connect()?;

            // Check VRF status
            let vrf_output = device.send_command("show vrf")?;
            println!("VRF Status:\n{}", vrf_output);

            device.close()?;
            Ok(())
        }
    }

    // Test NXOS-specific FEX feature
    #[test]
    fn test_nxos_fex_status() -> Result<(), NetsshError> {
        if use_mock_devices() {
            // Use mock device
            let mock_device = NxosSpecificMockDevice::new("admin", "admin");
            let port = mock_device.port();

            let mut config = setup_device_config();
            config.host = "127.0.0.1".to_string();
            config.port = Some(port);

            let mut device = DeviceFactory::create_device(&config)?;
            device.connect()?;

            // Check FEX status
            let fex_output = device.send_command("show fex")?;

            // Verify FEX information
            assert!(fex_output.contains("FEX-100"));
            assert!(fex_output.contains("FEX-101"));
            assert!(fex_output.contains("Online"));

            device.close()?;
            Ok(())
        } else {
            // Use real device
            let config = setup_device_config();
            let mut device = DeviceFactory::create_device(&config)?;
            device.connect()?;

            // Check FEX status (might not be present on all devices)
            let fex_output = device.send_command("show fex")?;
            println!("FEX Status (if configured):\n{}", fex_output);

            device.close()?;
            Ok(())
        }
    }

    // Test NXOS-specific interface configuration
    #[test]
    fn test_nxos_interface_configuration() -> Result<(), NetsshError> {
        if use_mock_devices() {
            // Use mock device
            let mock_device = NxosSpecificMockDevice::new("admin", "admin");
            let port = mock_device.port();

            let mut config = setup_device_config();
            config.host = "127.0.0.1".to_string();
            config.port = Some(port);

            let mut device = DeviceFactory::create_device(&config)?;
            device.connect()?;

            // Configure an interface with NXOS-specific commands
            let config_commands = vec![
                "interface Ethernet1/1",
                "description Test Interface",
                "switchport",
                "switchport mode access",
                "switchport access vlan 100",
                "no shutdown",
                "exit",
            ];

            device.send_config_commands(&config_commands)?;

            // Verify interface configuration
            let interface_config =
                device.send_command("show running-config interface Ethernet1/1")?;
            assert!(interface_config.contains("description Test Interface"));
            assert!(interface_config.contains("switchport access vlan 100"));

            // Check interface status
            let interface_status = device.send_command("show interface brief")?;
            assert!(interface_status.contains("Eth1/1"));

            device.close()?;
            Ok(())
        } else {
            // Use real device
            let config = setup_device_config();
            let mut device = DeviceFactory::create_device(&config)?;
            device.connect()?;

            // Check interface status without making changes
            let interface_status = device.send_command("show interface brief")?;
            println!("Interface Status:\n{}", interface_status);

            device.close()?;
            Ok(())
        }
    }
}
