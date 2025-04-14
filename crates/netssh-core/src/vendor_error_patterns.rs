use crate::device_connection::DeviceType;
use crate::error::NetsshError;
use lazy_static::lazy_static;
use regex::Regex;
use tracing::debug;

lazy_static! {
    // Cisco IOS error patterns
    pub static ref CISCO_IOS_ERRORS: Vec<Regex> = vec![
        Regex::new(r"% (?:Invalid|Incomplete|Ambiguous) (?:input|command)").unwrap(),
        Regex::new(r"% Error in authentication").unwrap(),
        Regex::new(r"% Bad passwords").unwrap(),
        Regex::new(r"% Unknown command").unwrap(),
        Regex::new(r"% Unrecognized command").unwrap(),
        Regex::new(r"% (?:Error|Not) (?:opening|possible)").unwrap(),
    ];

    // Cisco IOS-XR error patterns
    pub static ref CISCO_XR_ERRORS: Vec<Regex> = vec![
        Regex::new(r"% (?:Invalid|Incomplete|Ambiguous) (?:input|command)").unwrap(),
        Regex::new(r"% No matching").unwrap(),
        Regex::new(r"% Error in authentication").unwrap(),
        Regex::new(r"% Not allowed").unwrap(),
        Regex::new(r"error: .*").unwrap(),
    ];

    // Cisco NX-OS error patterns
    pub static ref CISCO_NXOS_ERRORS: Vec<Regex> = vec![
        Regex::new(r"% (?:Invalid|Incomplete|Ambiguous) (?:input|command)").unwrap(),
        Regex::new(r"% No matching").unwrap(),
        Regex::new(r"% Permission denied").unwrap(),
        Regex::new(r"% Error: ").unwrap(),
        Regex::new(r"ERROR: ").unwrap(),
        Regex::new(r"\^\s*\r?\n% Invalid parameter detected at '\^' marker\.").unwrap(),
        Regex::new(r"% Invalid parameter detected at '\^' marker\.").unwrap(),
    ];

    // Cisco ASA error patterns
    pub static ref CISCO_ASA_ERRORS: Vec<Regex> = vec![
        Regex::new(r"% (?:Invalid|Incomplete|Ambiguous) (?:input|command)").unwrap(),
        Regex::new(r"ERROR: ").unwrap(),
        Regex::new(r"% Error ").unwrap(),
        Regex::new(r"% Bad (?:password|secret)").unwrap(),
        Regex::new(r"% No such").unwrap(),
    ];

    // Juniper Junos error patterns
    pub static ref JUNIPER_JUNOS_ERRORS: Vec<Regex> = vec![
        Regex::new(r"(?:error|warning):").unwrap(),
        Regex::new(r"syntax error").unwrap(),
        Regex::new(r"syntax error, expecting").unwrap(),
        Regex::new(r"unknown command").unwrap(),
        Regex::new(r"invalid (?:command|input)").unwrap(),
        Regex::new(r"commit failed").unwrap(),
        Regex::new(r"configuration check-out failed").unwrap(),
        Regex::new(r"error:").unwrap(),
        Regex::new(r"\^\s*\r?\n").unwrap(),
        Regex::new(r"command is not valid").unwrap(),
        Regex::new(r"is ambiguous").unwrap(),
        Regex::new(r"is not").unwrap(),
        Regex::new(r"\^\s*\r?\nsyntax error, expecting <command>").unwrap(),
    ];
}

/// Returns the error patterns for the specified device type
pub fn get_error_patterns(device_type: &DeviceType) -> &'static Vec<Regex> {
    match device_type {
        DeviceType::CiscoIos => &CISCO_IOS_ERRORS,
        DeviceType::CiscoXr => &CISCO_XR_ERRORS,
        DeviceType::CiscoNxos => &CISCO_NXOS_ERRORS,
        DeviceType::CiscoAsa => &CISCO_ASA_ERRORS,
        DeviceType::JuniperJunos => &JUNIPER_JUNOS_ERRORS,
        DeviceType::Unknown => &CISCO_IOS_ERRORS, // Default to Cisco IOS for unknown devices
    }
}

/// Checks if the output matches any error pattern for the specified device type
pub fn check_for_errors(output: &str, device_type: &DeviceType) -> Option<String> {
    debug!("========================================================================================================================================================");
    debug!(
        "Checking for errors in output for device type: {:?}",
        device_type
    );
    debug!("Output to check: {}", output);

    let patterns = get_error_patterns(device_type);
    debug!("Number of error patterns to check: {}", patterns.len());

    for (i, pattern) in patterns.iter().enumerate() {
        debug!("Checking pattern {}: {:?}", i, pattern);
        if let Some(captures) = pattern.captures(output) {
            if let Some(matched) = captures.get(0) {
                let error = matched.as_str().to_string();
                debug!("Found error pattern match: {}", error);
                return Some(error);
            }
        }
    }

    debug!("No error patterns found in output");
    None
}

/// Checks command output against device-specific error patterns and converts matches to NetsshError
pub fn check_command_output(output: &str, device_type: &DeviceType) -> Result<(), NetsshError> {
    debug!("========================================================================================================================================================");
    debug!("Checking command output for device type: {:?}", device_type);
    debug!("========================================================================================================================================================");
    if let Some(error_match) = check_for_errors(output, device_type) {
        debug!("Found error pattern in command output: {}", error_match);
        Err(NetsshError::command_error_with_output(
            error_match,
            output.to_string(),
        ))
    } else {
        Ok(())
    }
}
