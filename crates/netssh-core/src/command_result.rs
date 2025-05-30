use crate::error::NetsshError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use indexmap::IndexMap;

/// Represents the output of a command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandOutput {
    /// Raw string output (when parsing not attempted or failed)
    Raw(String),
    /// Parsed structured data (when parsing succeeded)
    Parsed(Vec<IndexMap<String, serde_json::Value>>),
}

impl CommandOutput {
    /// Get the raw string output, regardless of whether it's parsed or not
    pub fn as_raw(&self) -> Option<&str> {
        match self {
            CommandOutput::Raw(s) => Some(s),
            CommandOutput::Parsed(_) => None,
        }
    }

    /// Get the parsed data if available
    pub fn as_parsed(&self) -> Option<&Vec<IndexMap<String, serde_json::Value>>> {
        match self {
            CommandOutput::Raw(_) => None,
            CommandOutput::Parsed(data) => Some(data),
        }
    }

    /// Check if the output contains parsed data
    pub fn is_parsed(&self) -> bool {
        matches!(self, CommandOutput::Parsed(_))
    }

    /// Get parsed data as JSON string
    pub fn parsed_as_json(&self) -> Option<Result<String, serde_json::Error>> {
        self.as_parsed().map(|data| serde_json::to_string_pretty(data))
    }

    /// Get a string representation of the output for display purposes
    pub fn to_display_string(&self) -> String {
        match self {
            CommandOutput::Raw(s) => s.clone(),
            CommandOutput::Parsed(data) => {
                serde_json::to_string_pretty(data).unwrap_or_else(|_| "Parse error".to_string())
            }
        }
    }
}

/// Represents the execution status of a command
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandStatus {
    /// Command executed successfully
    Success,
    /// Command execution failed
    Failed,
    /// Command execution timed out
    Timeout,
    /// Command execution was skipped (e.g., due to previous command failure)
    Skipped,
}

/// Represents the status of TextFSM parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParseStatus {
    /// Parsing was not attempted
    NotAttempted,
    /// Parsing succeeded
    Success,
    /// Parsing failed, raw output available
    Failed,
    /// No template found for platform/command
    NoTemplate,
}

/// Options for TextFSM parsing
#[derive(Debug, Clone)]
pub struct ParseOptions {
    /// Whether parsing is enabled
    pub enabled: bool,
    /// Optional custom template directory path
    pub template_dir: Option<String>,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            enabled: false,
            template_dir: None,
        }
    }
}

impl ParseOptions {
    /// Create new ParseOptions with parsing enabled
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            template_dir: None,
        }
    }

    /// Create new ParseOptions with parsing enabled and custom template directory
    pub fn with_template_dir<S: Into<String>>(template_dir: S) -> Self {
        Self {
            enabled: true,
            template_dir: Some(template_dir.into()),
        }
    }
}

/// Holds the result of executing a command on a device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    /// Optional device identifier (UUID from database, may not always be available)
    pub device_id: Option<String>,
    /// Device IP address
    pub device_ip: String,
    /// Device hostname
    pub hostname: String,
    /// Platform type (vendor/model)
    pub platform_type: String,
    /// The command that was executed
    pub command: String,
    /// Command output - either raw string or parsed structured data
    pub output: Option<CommandOutput>,
    /// Status of command execution
    pub status: CommandStatus,
    /// Error message if command failed
    pub error: Option<String>,
    /// Status of TextFSM parsing
    pub parse_status: ParseStatus,
    /// Error message if parsing failed
    pub parse_error: Option<String>,
}

impl CommandResult {
    /// Create a new CommandResult for a successful command execution
    pub fn success(
        device_id: Option<String>,
        device_ip: String,
        hostname: String,
        platform_type: String,
        command: String,
        output: String,
    ) -> Self {
        Self {
            device_id,
            device_ip,
            hostname,
            platform_type,
            command,
            output: Some(CommandOutput::Raw(output)),
            status: CommandStatus::Success,
            error: None,
            parse_status: ParseStatus::NotAttempted,
            parse_error: None,
        }
    }

    /// Create a new CommandResult for a failed command execution
    pub fn failure(
        device_id: Option<String>,
        device_ip: String,
        hostname: String,
        platform_type: String,
        command: String,
        output: String,
        error: String,
    ) -> Self {
        Self {
            device_id,
            device_ip,
            hostname,
            platform_type,
            command,
            output: Some(CommandOutput::Raw(output)),
            status: CommandStatus::Failed,
            error: Some(error),
            parse_status: ParseStatus::NotAttempted,
            parse_error: None,
        }
    }

    /// Create a new CommandResult for a timed out command execution
    pub fn timeout(
        device_id: Option<String>,
        device_ip: String,
        hostname: String,
        platform_type: String,
        command: String,
        error_message: String,
    ) -> Self {
        Self {
            device_id,
            device_ip,
            hostname,
            platform_type,
            command,
            output: None,
            status: CommandStatus::Timeout,
            error: Some(error_message),
            parse_status: ParseStatus::NotAttempted,
            parse_error: None,
        }
    }

    /// Create a new CommandResult for a skipped command execution
    pub fn skipped(
        device_id: Option<String>,
        device_ip: String,
        hostname: String,
        platform_type: String,
        command: String,
    ) -> Self {
        Self {
            device_id,
            device_ip,
            hostname,
            platform_type,
            command,
            output: None,
            status: CommandStatus::Skipped,
            error: None,
            parse_status: ParseStatus::NotAttempted,
            parse_error: None,
        }
    }

    /// Create a CommandResult from a command execution error
    pub fn from_error(
        device_id: Option<String>,
        device_ip: String,
        hostname: String,
        platform_type: String,
        command: String,
        error: NetsshError,
        output: Option<String>,
    ) -> Self {
        // Check if it's a timeout error
        match &error {
            NetsshError::Timeout { .. } => Self {
                device_id,
                device_ip,
                hostname,
                platform_type,
                command,
                output: output.map(CommandOutput::Raw),
                status: CommandStatus::Timeout,
                error: Some(format!("{}", error)),
                parse_status: ParseStatus::NotAttempted,
                parse_error: None,
            },
            NetsshError::CommandError(msg) => Self {
                device_id,
                device_ip,
                hostname,
                platform_type,
                command,
                output: output.map(CommandOutput::Raw),
                status: CommandStatus::Failed,
                error: Some(msg.clone()),
                parse_status: ParseStatus::NotAttempted,
                parse_error: None,
            },
            NetsshError::CommandErrorWithOutput {
                error_msg,
                output: cmd_output,
            } => Self {
                device_id,
                device_ip,
                hostname,
                platform_type,
                command,
                output: Some(CommandOutput::Raw(cmd_output.clone())),
                status: CommandStatus::Failed,
                error: Some(error_msg.clone()),
                parse_status: ParseStatus::NotAttempted,
                parse_error: None,
            },
            // All other errors get mapped to Failed status
            _ => Self {
                device_id,
                device_ip,
                hostname,
                platform_type,
                command,
                output: output.map(CommandOutput::Raw),
                status: CommandStatus::Failed,
                error: Some(format!("{}", error)),
                parse_status: ParseStatus::NotAttempted,
                parse_error: None,
            },
        }
    }

    /// Create a CommandResult with successful parsing
    pub fn success_with_parsing(
        device_id: Option<String>,
        device_ip: String,
        hostname: String,
        platform_type: String,
        command: String,
        parsed_data: Vec<IndexMap<String, serde_json::Value>>,
    ) -> Self {
        Self {
            device_id,
            device_ip,
            hostname,
            platform_type,
            command,
            output: Some(CommandOutput::Parsed(parsed_data)),
            status: CommandStatus::Success,
            error: None,
            parse_status: ParseStatus::Success,
            parse_error: None,
        }
    }

    /// Create a CommandResult with failed parsing (but successful command execution)
    pub fn success_with_parse_failure(
        device_id: Option<String>,
        device_ip: String,
        hostname: String,
        platform_type: String,
        command: String,
        output: String,
        parse_error: String,
    ) -> Self {
        Self {
            device_id,
            device_ip,
            hostname,
            platform_type,
            command,
            output: Some(CommandOutput::Raw(output)),
            status: CommandStatus::Failed, // Set to Failed when parsing fails
            error: None,
            parse_status: ParseStatus::Failed,
            parse_error: Some(parse_error),
        }
    }

    /// Create a CommandResult when no template was found for parsing
    pub fn success_with_no_template(
        device_id: Option<String>,
        device_ip: String,
        hostname: String,
        platform_type: String,
        command: String,
        output: String,
    ) -> Self {
        Self {
            device_id,
            device_ip,
            hostname,
            platform_type,
            command,
            output: Some(CommandOutput::Raw(output)),
            status: CommandStatus::Success,
            error: None,
            parse_status: ParseStatus::NoTemplate,
            parse_error: None,
        }
    }

    /// Check if parsing was successful
    pub fn is_parsed(&self) -> bool {
        self.parse_status == ParseStatus::Success
    }

    /// Check if parsing was attempted
    pub fn parse_attempted(&self) -> bool {
        self.parse_status != ParseStatus::NotAttempted
    }
}

/// Container for all results from a batch command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCommandResults {
    /// Map of device ID to a list of command results for that device
    pub results: HashMap<String, Vec<CommandResult>>,
    /// Total number of devices processed
    pub device_count: usize,
    /// Total number of commands executed
    pub command_count: usize,
    /// Number of successful commands
    pub success_count: usize,
    /// Number of failed commands
    pub failure_count: usize,
    /// Number of timed out commands
    pub timeout_count: usize,
    /// Number of skipped commands
    pub skipped_count: usize,
}

impl BatchCommandResults {
    /// Create a new empty BatchCommandResults
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
            device_count: 0,
            command_count: 0,
            success_count: 0,
            failure_count: 0,
            timeout_count: 0,
            skipped_count: 0,
        }
    }

    /// Add a command result to the batch results
    pub fn add_result(&mut self, result: CommandResult) {
        // Use device_ip as the key since device_id is now optional
        let device_key = result.device_ip.clone();

        // Update counters based on result status
        match result.status {
            CommandStatus::Success => self.success_count += 1,
            CommandStatus::Failed => self.failure_count += 1,
            CommandStatus::Timeout => self.timeout_count += 1,
            CommandStatus::Skipped => self.skipped_count += 1,
        }

        self.command_count += 1;

        // Add result to device's results
        self.results
            .entry(device_key)
            .or_insert_with(Vec::new)
            .push(result);

        // Update device count
        self.device_count = self.results.len();
    }

    /// Get all results for a specific device
    pub fn get_device_results(&self, device_id: &str) -> Option<&Vec<CommandResult>> {
        self.results.get(device_id)
    }

    /// Get all results for a specific command across all devices
    pub fn get_command_results(&self, command: &str) -> Vec<&CommandResult> {
        self.results
            .values()
            .flat_map(|results| results.iter())
            .filter(|result| result.command == command)
            .collect()
    }

    /// Filter results by status
    pub fn filter_by_status(&self, status: CommandStatus) -> Vec<&CommandResult> {
        self.results
            .values()
            .flat_map(|results| results.iter())
            .filter(|result| result.status == status)
            .collect()
    }

    /// Get successful results
    pub fn successful_results(&self) -> Vec<&CommandResult> {
        self.filter_by_status(CommandStatus::Success)
    }

    /// Get failed results
    pub fn failed_results(&self) -> Vec<&CommandResult> {
        self.filter_by_status(CommandStatus::Failed)
    }

    /// Get timed out results
    pub fn timeout_results(&self) -> Vec<&CommandResult> {
        self.filter_by_status(CommandStatus::Timeout)
    }

    /// Get skipped results
    pub fn skipped_results(&self) -> Vec<&CommandResult> {
        self.filter_by_status(CommandStatus::Skipped)
    }
}

/// Utility functions for working with batch command results
pub mod utils {
    use super::*;
    use serde_json;

    /// Convert batch results to JSON format
    pub fn to_json(results: &BatchCommandResults) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(results)
    }

    /// Convert batch results to CSV format
    pub fn to_csv(results: &BatchCommandResults) -> String {
        let mut csv = String::new();

        // Add header
        csv.push_str("device_ip,hostname,platform_type,command,status,parse_status\n");

        // Add rows
        for (_, device_results) in &results.results {
            for result in device_results {
                csv.push_str(&format!(
                    "{},{},{},{},{:?},{:?}\n",
                    result.device_ip,
                    result.hostname,
                    result.platform_type,
                    result.command.replace(",", "\\,"),
                    result.status,
                    result.parse_status
                ));
            }
        }

        csv
    }

    /// Group results by command
    pub fn group_by_command(results: &BatchCommandResults) -> HashMap<&str, Vec<&CommandResult>> {
        let mut grouped: HashMap<&str, Vec<&CommandResult>> = HashMap::new();

        for (_, device_results) in &results.results {
            for result in device_results {
                grouped
                    .entry(&result.command)
                    .or_insert_with(Vec::new)
                    .push(result);
            }
        }

        grouped
    }

    /// Group results by device
    pub fn group_by_device(results: &BatchCommandResults) -> HashMap<&str, Vec<&CommandResult>> {
        let mut grouped: HashMap<&str, Vec<&CommandResult>> = HashMap::new();

        for (device_id, device_results) in &results.results {
            for result in device_results {
                grouped
                    .entry(device_id)
                    .or_insert_with(Vec::new)
                    .push(result);
            }
        }

        grouped
    }

    /// Compare outputs across devices for the same command
    pub fn compare_outputs(
        results: &BatchCommandResults,
        command: &str,
    ) -> HashMap<String, Vec<String>> {
        let mut output_groups: HashMap<String, Vec<String>> = HashMap::new();

        for result in results.get_command_results(command) {
            if result.status == CommandStatus::Success {
                if let Some(output) = &result.output {
                    // Convert output to string for comparison
                    let output_str = match output {
                        CommandOutput::Raw(s) => s.clone(),
                        CommandOutput::Parsed(data) => {
                            // For parsed data, use JSON representation for comparison
                            serde_json::to_string_pretty(data).unwrap_or_else(|_| "Parse error".to_string())
                        }
                    };
                    output_groups
                        .entry(output_str)
                        .or_insert_with(Vec::new)
                        .push(result.device_ip.clone());
                }
            }
        }

        output_groups
    }

    /// Format results as a table
    pub fn format_as_table(results: &BatchCommandResults) -> String {
        let mut table = String::new();

        // Add header
        table.push_str("+-----------------+-----------------+--------------------------------+--------+-------------+------------------+\n");
        table.push_str("| Device IP       | Hostname        | Command                        | Status | Parse Status| Error            |\n");
        table.push_str("+-----------------+-----------------+--------------------------------+--------+-------------+------------------+\n");

        // Add rows
        for (_, device_results) in &results.results {
            for result in device_results {
                let status = match result.status {
                    CommandStatus::Success => "SUCCESS",
                    CommandStatus::Failed => "FAILED",
                    CommandStatus::Timeout => "TIMEOUT",
                    CommandStatus::Skipped => "SKIPPED",
                };

                let parse_status = match result.parse_status {
                    ParseStatus::Success => "PARSED",
                    ParseStatus::Failed => "PARSE_FAIL",
                    ParseStatus::NoTemplate => "NO_TEMPLATE",
                    ParseStatus::NotAttempted => "NOT_PARSED",
                };

                // Truncate long values for table display
                let device_ip = truncate(&result.device_ip, 15);
                let hostname = truncate(&result.hostname, 15);
                let command = truncate(&result.command, 30);
                let error = match &result.error {
                    Some(err) => truncate(err, 16),
                    None => "".to_string(),
                };

                table.push_str(&format!(
                    "| {:<15} | {:<15} | {:<30} | {:<6} | {:<11} | {:<16} |\n",
                    device_ip,
                    hostname,
                    command,
                    status,
                    parse_status,
                    error
                ));
            }
        }

        // Add footer
        table.push_str("+-----------------+-----------------+--------------------------------+--------+-------------+------------------+\n");

        // Add summary
        table.push_str(&format!(
            "Summary: {} devices, {} commands ({} success, {} failed, {} timeout, {} skipped)\n",
            results.device_count,
            results.command_count,
            results.success_count,
            results.failure_count,
            results.timeout_count,
            results.skipped_count
        ));

        table
    }

    // Helper function to truncate strings for table display
    fn truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[0..max_len - 3])
        }
    }
}
