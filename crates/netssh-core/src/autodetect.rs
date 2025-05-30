use regex::Regex;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use tracing::{debug, info};

use crate::base_connection::BaseConnection;
use crate::device_connection::DeviceConfig;
use crate::error::NetsshError;

lazy_static! {
    // Common vendor identification patterns
    static ref CISCO_IOS_PATTERN: Regex = Regex::new(r"(?i)IOS Software|Cisco IOS Software").unwrap();
    static ref CISCO_XR_PATTERN: Regex = Regex::new(r"(?i)IOS XR Software|Cisco IOS XR Software").unwrap();
    static ref CISCO_NXOS_PATTERN: Regex = Regex::new(r"(?i)NX-OS|Cisco Nexus Operating System").unwrap();
    static ref CISCO_ASA_PATTERN: Regex = Regex::new(r"(?i)Adaptive Security Appliance|ASA").unwrap();
    static ref JUNIPER_JUNOS_PATTERN: Regex = Regex::new(r"(?i)JUNOS|Juniper Networks").unwrap();
}

/// Device type identification based on the output
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceType {
    CiscoIOS,
    CiscoXR,
    CiscoNXOS,
    CiscoASA,
    JuniperJunos,
    Unknown,
}

/// Result of the device detection process
#[derive(Debug, Clone)]
pub struct DetectionResult {
    /// Detected device type
    pub device_type: DeviceType,
    /// Output from the detection process
    pub output: String,
    /// Base prompt observed during detection
    pub base_prompt: Option<String>,
}

/// Struct to store device mapping information for autodetection
#[derive(Debug, Clone)]
struct DeviceMapping {
    cmd: String,
    search_patterns: Vec<String>,
    priority: u8,
    dispatch: Option<String>, // Optional dispatch method name
}

/// The SSHDetect struct tries to automatically guess the device type
/// running on the SSH remote end.
pub struct SSHDetect {
    connection: BaseConnection,
    potential_matches: HashMap<String, u8>,
    results_cache: HashMap<String, String>,
}

/// Create a HashMap of device mappings for autodetection
fn create_device_mapper() -> HashMap<String, DeviceMapping> {
    let mut mapper = HashMap::new();

    // Alcatel AOS
    mapper.insert(
        "alcatel_aos".to_string(),
        DeviceMapping {
            cmd: "show system".to_string(),
            search_patterns: vec![r"Alcatel-Lucent".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // Alcatel SROS
    mapper.insert(
        "alcatel_sros".to_string(),
        DeviceMapping {
            cmd: "show version".to_string(),
            search_patterns: vec!["Nokia".to_string(), "Alcatel".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // Allied Telesis AW+
    mapper.insert(
        "allied_telesis_awplus".to_string(),
        DeviceMapping {
            cmd: "show version".to_string(),
            search_patterns: vec![r"AlliedWare Plus".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // Apresia AEOS
    mapper.insert(
        "apresia_aeos".to_string(),
        DeviceMapping {
            cmd: "show system".to_string(),
            search_patterns: vec![r"Apresia".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // Arista EOS
    mapper.insert(
        "arista_eos".to_string(),
        DeviceMapping {
            cmd: "show version".to_string(),
            search_patterns: vec![r"Arista".to_string(), r"vEOS".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // Aruba AOSCX
    mapper.insert(
        "aruba_aoscx".to_string(),
        DeviceMapping {
            cmd: "show version".to_string(),
            search_patterns: vec![r"ArubaOS-CX".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // Ciena SAOS
    mapper.insert(
        "ciena_saos".to_string(),
        DeviceMapping {
            cmd: "software show".to_string(),
            search_patterns: vec![r"saos".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // Cisco ASA
    mapper.insert(
        "cisco_asa".to_string(),
        DeviceMapping {
            cmd: "show version".to_string(),
            search_patterns: vec![
                r"Cisco Adaptive Security Appliance".to_string(),
                r"Cisco ASA".to_string(),
            ],
            priority: 99,
            dispatch: None,
        },
    );

    // Cisco FTD
    mapper.insert(
        "cisco_ftd".to_string(),
        DeviceMapping {
            cmd: "show version".to_string(),
            search_patterns: vec![r"Cisco Firepower".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // Cisco IOS
    mapper.insert(
        "cisco_ios".to_string(),
        DeviceMapping {
            cmd: "show version".to_string(),
            search_patterns: vec![
                r"Cisco IOS Software".to_string(),
                r"Cisco Internetwork Operating System Software".to_string(),
            ],
            priority: 95,
            dispatch: None,
        },
    );

    // Cisco XE
    mapper.insert(
        "cisco_xe".to_string(),
        DeviceMapping {
            cmd: "show version".to_string(),
            search_patterns: vec![r"Cisco IOS XE Software".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // Cisco NXOS
    mapper.insert(
        "cisco_nxos".to_string(),
        DeviceMapping {
            cmd: "show version".to_string(),
            search_patterns: vec![
                r"Cisco Nexus Operating System".to_string(),
                r"NX-OS".to_string(),
            ],
            priority: 99,
            dispatch: None,
        },
    );

    // Cisco XR
    mapper.insert(
        "cisco_xr".to_string(),
        DeviceMapping {
            cmd: "show version".to_string(),
            search_patterns: vec![r"Cisco IOS XR".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // Cisco XR alternative
    mapper.insert(
        "cisco_xr_2".to_string(),
        DeviceMapping {
            cmd: "show version brief".to_string(),
            search_patterns: vec![r"Cisco IOS XR".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // Dell Force10
    mapper.insert(
        "dell_force10".to_string(),
        DeviceMapping {
            cmd: "show version".to_string(),
            search_patterns: vec![r"Real Time Operating System Software".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // Dell OS9
    mapper.insert(
        "dell_os9".to_string(),
        DeviceMapping {
            cmd: "show system".to_string(),
            search_patterns: vec![
                r"Dell Application Software Version:  9".to_string(),
                r"Dell Networking OS Version : 9".to_string(),
            ],
            priority: 99,
            dispatch: None,
        },
    );

    // Dell OS10
    mapper.insert(
        "dell_os10".to_string(),
        DeviceMapping {
            cmd: "show version".to_string(),
            search_patterns: vec![r"Dell EMC Networking OS10.Enterprise".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // Dell PowerConnect
    mapper.insert(
        "dell_powerconnect".to_string(),
        DeviceMapping {
            cmd: "show system".to_string(),
            search_patterns: vec![r"PowerConnect".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // F5 TMSH
    mapper.insert(
        "f5_tmsh".to_string(),
        DeviceMapping {
            cmd: "show sys version".to_string(),
            search_patterns: vec![r"BIG-IP".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // F5 Linux
    mapper.insert(
        "f5_linux".to_string(),
        DeviceMapping {
            cmd: "cat /etc/issue".to_string(),
            search_patterns: vec![r"BIG-IP".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // HP Comware
    mapper.insert(
        "hp_comware".to_string(),
        DeviceMapping {
            cmd: "display version".to_string(),
            search_patterns: vec!["HPE Comware".to_string(), "HP Comware".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // HP ProCurve
    mapper.insert(
        "hp_procurve".to_string(),
        DeviceMapping {
            cmd: "show version".to_string(),
            search_patterns: vec![r"Image stamp.*/code/build".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // Huawei
    mapper.insert(
        "huawei".to_string(),
        DeviceMapping {
            cmd: "display version".to_string(),
            search_patterns: vec![
                r"Huawei Technologies".to_string(),
                r"Huawei Versatile Routing Platform Software".to_string(),
            ],
            priority: 99,
            dispatch: None,
        },
    );

    // Juniper JunOS
    mapper.insert(
        "juniper_junos".to_string(),
        DeviceMapping {
            cmd: "show version".to_string(),
            search_patterns: vec![
                r"JUNOS Software Release".to_string(),
                r"JUNOS .+ Software".to_string(),
                r"JUNOS OS Kernel".to_string(),
                r"JUNOS Base Version".to_string(),
            ],
            priority: 99,
            dispatch: None,
        },
    );

    // Linux
    mapper.insert(
        "linux".to_string(),
        DeviceMapping {
            cmd: "uname -a".to_string(),
            search_patterns: vec![r"Linux".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // Ericsson IPOS
    mapper.insert(
        "ericsson_ipos".to_string(),
        DeviceMapping {
            cmd: "show version".to_string(),
            search_patterns: vec![r"Ericsson IPOS Version".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // Extreme EXOS
    mapper.insert(
        "extreme_exos".to_string(),
        DeviceMapping {
            cmd: "show version".to_string(),
            search_patterns: vec![r"ExtremeXOS".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // Extreme NetIron
    mapper.insert(
        "extreme_netiron".to_string(),
        DeviceMapping {
            cmd: "show version".to_string(),
            search_patterns: vec![r"(NetIron|MLX)".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // Extreme SLX
    mapper.insert(
        "extreme_slx".to_string(),
        DeviceMapping {
            cmd: "show version".to_string(),
            search_patterns: vec![r"SLX-OS Operating System".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // Extreme Tierra
    mapper.insert(
        "extreme_tierra".to_string(),
        DeviceMapping {
            cmd: "show version".to_string(),
            search_patterns: vec![r"TierraOS Software".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // Ubiquiti EdgeSwitch
    mapper.insert(
        "ubiquiti_edgeswitch".to_string(),
        DeviceMapping {
            cmd: "show version".to_string(),
            search_patterns: vec![r"EdgeSwitch".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // Cisco WLC (legacy)
    mapper.insert(
        "cisco_wlc".to_string(),
        DeviceMapping {
            cmd: "".to_string(),
            search_patterns: vec![r"CISCO_WLC".to_string()],
            priority: 99,
            dispatch: Some("remote_version".to_string()),
        },
    );

    // Cisco WLC 8.5+
    mapper.insert(
        "cisco_wlc_85".to_string(),
        DeviceMapping {
            cmd: "show inventory".to_string(),
            search_patterns: vec![r"Cisco.*Wireless.*Controller".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // Mellanox MLNXOS
    mapper.insert(
        "mellanox_mlnxos".to_string(),
        DeviceMapping {
            cmd: "show version".to_string(),
            search_patterns: vec![r"Onyx".to_string(), r"SX_PPC_M460EX".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // Yamaha
    mapper.insert(
        "yamaha".to_string(),
        DeviceMapping {
            cmd: "show copyright".to_string(),
            search_patterns: vec![r"Yamaha Corporation".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // Fortinet
    mapper.insert(
        "fortinet".to_string(),
        DeviceMapping {
            cmd: "get system status".to_string(),
            search_patterns: vec![r"FortiOS".to_string(), r"FortiGate".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // Palo Alto PanOS
    mapper.insert(
        "paloalto_panos".to_string(),
        DeviceMapping {
            cmd: "show system info".to_string(),
            search_patterns: vec![r"model:\s+PA".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // Supermicro SMIS
    mapper.insert(
        "supermicro_smis".to_string(),
        DeviceMapping {
            cmd: "show system info".to_string(),
            search_patterns: vec![r"Super Micro Computer".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // FlexVNF
    mapper.insert(
        "flexvnf".to_string(),
        DeviceMapping {
            cmd: "show system package-info".to_string(),
            search_patterns: vec![r"Versa FlexVNF".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // Cisco Viptela
    mapper.insert(
        "cisco_viptela".to_string(),
        DeviceMapping {
            cmd: "show system status".to_string(),
            search_patterns: vec![r"Viptela, Inc".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // OneAccess OneOS
    mapper.insert(
        "oneaccess_oneos".to_string(),
        DeviceMapping {
            cmd: "show version".to_string(),
            search_patterns: vec![r"OneOS".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // Netgear ProSAFE
    mapper.insert(
        "netgear_prosafe".to_string(),
        DeviceMapping {
            cmd: "show version".to_string(),
            search_patterns: vec![r"ProSAFE".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    // Huawei SmartAX
    mapper.insert(
        "huawei_smartax".to_string(),
        DeviceMapping {
            cmd: "display version".to_string(),
            search_patterns: vec![r"Huawei Integrated Access Software".to_string()],
            priority: 99,
            dispatch: None,
        },
    );

    mapper
}

impl SSHDetect {
    /// Creates a new SSHDetect instance with the provided device configuration
    pub fn new(config: &DeviceConfig) -> Result<Self, NetsshError> {
        if config.device_type != "autodetect" {
            return Err(NetsshError::InvalidOperation(
                "The device_type must be 'autodetect' for SSHDetect".to_string(),
            ));
        }

        let mut base_connection = BaseConnection::new()?;
        base_connection.connect(
            Some(&config.host),
            Some(&config.username),
            config.password.as_deref(),
            config.port,
            config.timeout,
        )?;

        // Add additional sleep to let the login complete
        thread::sleep(Duration::from_secs(3));

        // Update clear_buffer call with all required parameters
        let _ = base_connection.clear_buffer(None, None, None, None, None, None)?;

        Ok(SSHDetect {
            connection: base_connection,
            potential_matches: HashMap::new(),
            results_cache: HashMap::new(),
        })
    }

    /// Try to detect the device type based on the command outputs and patterns
    pub fn autodetect(&mut self) -> Result<Option<String>, NetsshError> {
        let device_mapper = create_device_mapper();

        self.connection.set_base_prompt(None, None, None, None)?;

        for (device_type, mapping) in device_mapper.iter() {
            debug!("Attempting to detect device type: {}", device_type);

            // Check dispatch method
            let accuracy = if let Some(dispatch) = &mapping.dispatch {
                match dispatch.as_str() {
                    "remote_version" => {
                        self.autodetect_remote_version(&mapping.search_patterns, mapping.priority)?
                    }
                    _ => {
                        debug!(
                            "Unknown dispatch method {}, using standard method",
                            dispatch
                        );
                        if !mapping.cmd.is_empty() {
                            self.autodetect_std(
                                &mapping.cmd,
                                &mapping.search_patterns,
                                mapping.priority,
                            )?
                        } else {
                            0 // Skip empty commands with unknown dispatch
                        }
                    }
                }
            } else {
                // Skip empty commands with standard dispatch
                if mapping.cmd.is_empty() {
                    continue;
                }

                self.autodetect_std(&mapping.cmd, &mapping.search_patterns, mapping.priority)?
            };

            if accuracy > 0 {
                self.potential_matches.insert(device_type.clone(), accuracy);
                if accuracy >= 99 {
                    // We're confident enough, stop the detection
                    info!(
                        "High confidence match found for device type: {}",
                        device_type
                    );

                    // Handle special cases like in the Python example
                    if device_type == "cisco_wlc_85" {
                        info!("Detected cisco_wlc_85, using driver cisco_wlc");
                        return Ok(Some("cisco_wlc".to_string()));
                    } else if device_type == "cisco_xr_2" {
                        info!("Detected cisco_xr_2, using driver cisco_xr");
                        return Ok(Some("cisco_xr".to_string()));
                    }

                    return Ok(Some(device_type.clone()));
                }
            }
        }

        // If we have potential matches, return the one with the highest accuracy
        if !self.potential_matches.is_empty() {
            let best_match = self
                .potential_matches
                .iter()
                .max_by_key(|&(_, accuracy)| accuracy)
                .map(|(device_type, _)| {
                    let device_type = device_type.clone();

                    // Check for special cases in best match
                    if device_type == "cisco_wlc_85" {
                        "cisco_wlc".to_string()
                    } else if device_type == "cisco_xr_2" {
                        "cisco_xr".to_string()
                    } else {
                        device_type
                    }
                });

            return Ok(best_match);
        }

        // No matches found
        Ok(None)
    }

    /// Detect device based on the SSH server's remote version
    fn autodetect_remote_version(
        &self,
        search_patterns: &[String],
        priority: u8,
    ) -> Result<u8, NetsshError> {
        let invalid_responses = [r"^$"];

        // Get remote version from the SSH connection
        let remote_version = match self.connection.get_remote_version() {
            Some(version) => version,
            None => {
                debug!("Couldn't get remote version");
                return Ok(0);
            }
        };

        // Check for invalid responses
        for pattern in invalid_responses.iter() {
            if Regex::new(pattern).unwrap().is_match(&remote_version) {
                return Ok(0);
            }
        }

        // Check for matching patterns
        for pattern in search_patterns {
            if Regex::new(pattern).unwrap().is_match(&remote_version) {
                debug!("Found match in remote version: {}", pattern);
                return Ok(priority);
            }
        }

        Ok(0)
    }

    /// Attempt to detect device type based on sending a command and matching patterns
    fn autodetect_std(
        &mut self,
        cmd: &str,
        search_patterns: &[String],
        priority: u8,
    ) -> Result<u8, NetsshError> {
        let invalid_responses = [
            r"% Invalid input detected",
            r"syntax error, expecting",
            r"Error: Unrecognized command",
            r"%Error",
            r"command not found",
            r"Syntax Error: unexpected argument",
            r"% Unrecognized command found at",
            r"% Unknown command, the error locates at",
        ];

        // Check cache first
        if let Some(cached_response) = self.results_cache.get(cmd) {
            debug!("Using cached response for command: {}", cmd);
            return self.check_patterns(
                cached_response,
                search_patterns,
                &invalid_responses,
                priority,
            );
        }

        // Send the command and get the response
        debug!("Sending command: {}", cmd);
        let response = match self.connection.send_command_internal(cmd, None, None, None, None, None, None, None) {
            Ok(resp) => resp,
            Err(e) => {
                debug!("Error sending command: {}", e);
                return Ok(0);
            }
        };

        // Cache the response
        self.results_cache.insert(cmd.to_string(), response.clone());

        // Check patterns
        self.check_patterns(&response, search_patterns, &invalid_responses, priority)
    }

    /// Helper function to check patterns in the response
    fn check_patterns(
        &self,
        response: &str,
        search_patterns: &[String],
        invalid_responses: &[&str],
        priority: u8,
    ) -> Result<u8, NetsshError> {
        // Check for error conditions
        for pattern in invalid_responses {
            if Regex::new(pattern).unwrap().is_match(response) {
                return Ok(0);
            }
        }

        // Look for matching patterns
        for pattern in search_patterns {
            if Regex::new(pattern).unwrap().is_match(response) {
                return Ok(priority);
            }
        }

        Ok(0)
    }

    /// Close the connection when done
    pub fn disconnect(&mut self) -> Result<(), NetsshError> {
        // First close the channel
        if let Err(e) = self.connection.channel.close() {
            debug!("Error closing channel: {}", e);
            // Continue anyway to also close the session
        }

        // Then close the session by dropping it
        if let Some(session) = self.connection.session.take() {
            // Dropping the session will close the connection
            drop(session);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Test the device mapper creation
    #[test]
    fn test_create_device_mapper() {
        let mapper = create_device_mapper();

        // Check if we have the expected device types
        assert!(mapper.contains_key("cisco_ios"));
        assert!(mapper.contains_key("cisco_xe"));
        assert!(mapper.contains_key("cisco_xr"));
        assert!(mapper.contains_key("cisco_nxos"));
        assert!(mapper.contains_key("cisco_asa"));
        assert!(mapper.contains_key("juniper_junos"));
        assert!(mapper.contains_key("f5_tmsh"));
        assert!(mapper.contains_key("hp_comware"));
        assert!(mapper.contains_key("huawei"));
        assert!(mapper.contains_key("linux"));
        assert!(mapper.contains_key("fortinet"));

        // Check specific pattern exists
        let cisco_ios = mapper.get("cisco_ios").unwrap();
        assert_eq!(cisco_ios.cmd, "show version");
        assert!(cisco_ios
            .search_patterns
            .contains(&r"Cisco IOS Software".to_string()));

        // Check for dispatch method in WLC
        let cisco_wlc = mapper.get("cisco_wlc").unwrap();
        assert_eq!(cisco_wlc.cmd, "");
        assert_eq!(cisco_wlc.dispatch, Some("remote_version".to_string()));
        assert!(cisco_wlc
            .search_patterns
            .contains(&r"CISCO_WLC".to_string()));
    }

    // Test the pattern checking functionality
    #[test]
    fn test_check_patterns() {
        let ssh_detect = SSHDetect {
            connection: BaseConnection::new().unwrap(),
            potential_matches: HashMap::new(),
            results_cache: HashMap::new(),
        };

        // Test valid pattern match
        let response = "Cisco IOS Software, C2600 Software (C2600-ADVENTERPRISEK9-M), Version 12.4(15)T14, RELEASE SOFTWARE (fc2)";
        let patterns = vec![r"Cisco IOS Software".to_string()];
        let invalid_patterns = [r"% Invalid input detected"];

        let result = ssh_detect
            .check_patterns(response, &patterns, &invalid_patterns, 95)
            .unwrap();
        assert_eq!(result, 95);

        // Test error pattern match
        let error_response = "% Invalid input detected at '^' marker.";
        let result = ssh_detect
            .check_patterns(error_response, &patterns, &invalid_patterns, 95)
            .unwrap();
        assert_eq!(result, 0);

        // Test no match
        let no_match_response = "JunOS 12.1R1.9";
        let result = ssh_detect
            .check_patterns(no_match_response, &patterns, &invalid_patterns, 95)
            .unwrap();
        assert_eq!(result, 0);
    }

    // Test the special case handling
    #[test]
    fn test_special_case_handling() {
        let mut ssh_detect = SSHDetect {
            connection: BaseConnection::new().unwrap(),
            potential_matches: HashMap::new(),
            results_cache: HashMap::new(),
        };

        // Add a high-confidence cisco_wlc_85 match
        ssh_detect
            .potential_matches
            .insert("cisco_wlc_85".to_string(), 99);

        // Manually trigger the best match selection
        let matches = ssh_detect.potential_matches.clone();
        let best_match =
            matches
                .iter()
                .max_by_key(|&(_, accuracy)| accuracy)
                .map(|(device_type, _)| {
                    let device_type = device_type.clone();

                    // Check for special cases in best match
                    if device_type == "cisco_wlc_85" {
                        "cisco_wlc".to_string()
                    } else if device_type == "cisco_xr_2" {
                        "cisco_xr".to_string()
                    } else {
                        device_type
                    }
                });

        assert_eq!(best_match, Some("cisco_wlc".to_string()));

        // Test cisco_xr_2 special case
        ssh_detect.potential_matches.clear();
        ssh_detect
            .potential_matches
            .insert("cisco_xr_2".to_string(), 99);

        let matches = ssh_detect.potential_matches.clone();
        let best_match =
            matches
                .iter()
                .max_by_key(|&(_, accuracy)| accuracy)
                .map(|(device_type, _)| {
                    let device_type = device_type.clone();

                    // Check for special cases in best match
                    if device_type == "cisco_wlc_85" {
                        "cisco_wlc".to_string()
                    } else if device_type == "cisco_xr_2" {
                        "cisco_xr".to_string()
                    } else {
                        device_type
                    }
                });

        assert_eq!(best_match, Some("cisco_xr".to_string()));
    }
}
