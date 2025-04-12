use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::error::NetsshError;

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

/// Holds the result of executing a command on a device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    /// Identifier for the device (hostname or IP)
    pub device_id: String,
    /// Device type (vendor/model)
    pub device_type: String,
    /// The command that was executed
    pub command: String,
    /// Output text from the command
    pub output: Option<String>,
    /// Time when command execution started
    pub start_time: DateTime<Utc>,
    /// Time when command execution ended
    pub end_time: DateTime<Utc>,
    /// Duration of command execution in milliseconds
    pub duration_ms: u64,
    /// Status of command execution
    pub status: CommandStatus,
    /// Error message if command failed
    pub error: Option<String>,
}

impl CommandResult {
    /// Create a new CommandResult for a successful command execution
    pub fn success(
        device_id: String,
        device_type: String,
        command: String,
        output: String,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Self {
        let duration = end_time.signed_duration_since(start_time);
        let duration_ms = duration.num_milliseconds() as u64;

        Self {
            device_id,
            device_type,
            command,
            output: Some(output),
            start_time,
            end_time,
            duration_ms,
            status: CommandStatus::Success,
            error: None,
        }
    }

    /// Create a new CommandResult for a failed command execution
    pub fn failure(
        device_id: String,
        device_type: String,
        command: String,
        output: String,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        error: String,
    ) -> Self {
        let duration = end_time.signed_duration_since(start_time);
        let duration_ms = duration.num_milliseconds() as u64;

        Self {
            device_id,
            device_type,
            command,
            output: Some(output),
            start_time,
            end_time,
            duration_ms,
            status: CommandStatus::Failed,
            error: Some(error),
        }
    }

    /// Create a new CommandResult for a timed out command execution
    pub fn timeout(
        device_id: String,
        device_type: String,
        command: String,
        start_time: DateTime<Utc>,
    ) -> Self {
        let end_time = Utc::now();
        let duration = end_time.signed_duration_since(start_time);
        let duration_ms = duration.num_milliseconds() as u64;

        Self {
            device_id,
            device_type,
            command,
            output: None,
            start_time,
            end_time,
            duration_ms,
            status: CommandStatus::Timeout,
            error: Some(format!(
                "Command execution timed out after {} ms",
                duration_ms
            )),
        }
    }

    /// Create a new CommandResult for a skipped command execution
    pub fn skipped(device_id: String, device_type: String, command: String) -> Self {
        let now = Utc::now();

        Self {
            device_id,
            device_type,
            command,
            output: None,
            start_time: now,
            end_time: now,
            duration_ms: 0,
            status: CommandStatus::Skipped,
            error: None,
        }
    }

    /// Create a CommandResult from a command execution error
    pub fn from_error(
        device_id: String,
        device_type: String,
        command: String,
        error: NetsshError,
        start_time: DateTime<Utc>,
        output: Option<String>,
    ) -> Self {
        let end_time = Utc::now();
        let duration = end_time.signed_duration_since(start_time);
        let duration_ms = duration.num_milliseconds() as u64;

        // Check if it's a timeout error
        match &error {
            NetsshError::Timeout { .. } => Self {
                device_id,
                device_type,
                command,
                output,
                start_time,
                end_time,
                duration_ms,
                status: CommandStatus::Timeout,
                error: Some(format!("{}", error)),
            },
            NetsshError::CommandError(msg) => Self {
                device_id,
                device_type,
                command,
                output,
                start_time,
                end_time,
                duration_ms,
                status: CommandStatus::Failed,
                error: Some(msg.clone()),
            },
            // All other errors get mapped to Failed status
            _ => Self {
                device_id,
                device_type,
                command,
                output,
                start_time,
                end_time,
                duration_ms,
                status: CommandStatus::Failed,
                error: Some(format!("{}", error)),
            },
        }
    }

    /// Create a CommandResult from a command execution result
    pub fn from_result(
        device_id: String,
        device_type: String,
        command: String,
        result: Result<String, NetsshError>,
        start_time: DateTime<Utc>,
    ) -> Self {
        match result {
            Ok(output) => Self::success(device_id, device_type, command, output, start_time, Utc::now()),
            Err(error) => Self::from_error(device_id, device_type, command, error, start_time, None),
        }
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
    /// Time when batch execution started
    pub start_time: DateTime<Utc>,
    /// Time when batch execution ended
    pub end_time: DateTime<Utc>,
    /// Duration of batch execution in milliseconds
    pub duration_ms: u64,
}

impl BatchCommandResults {
    /// Create a new empty BatchCommandResults
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            results: HashMap::new(),
            device_count: 0,
            command_count: 0,
            success_count: 0,
            failure_count: 0,
            timeout_count: 0,
            skipped_count: 0,
            start_time: now,
            end_time: now,
            duration_ms: 0,
        }
    }

    /// Add a command result to the batch results
    pub fn add_result(&mut self, result: CommandResult) {
        let device_id = result.device_id.clone();

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
            .entry(device_id)
            .or_insert_with(Vec::new)
            .push(result);

        // Update device count
        self.device_count = self.results.len();
    }

    /// Complete the batch results with timing information
    pub fn complete(&mut self) {
        self.end_time = Utc::now();
        let duration = self.end_time.signed_duration_since(self.start_time);
        self.duration_ms = duration.num_milliseconds() as u64;
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
        csv.push_str("device_id,device_type,command,status,duration_ms,start_time,end_time\n");

        // Add rows
        for (_, device_results) in &results.results {
            for result in device_results {
                csv.push_str(&format!(
                    "{},{},{},{:?},{},{},{}\n",
                    result.device_id,
                    result.device_type,
                    result.command.replace(",", "\\,"),
                    result.status,
                    result.duration_ms,
                    result.start_time,
                    result.end_time
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
                    output_groups
                        .entry(output.clone())
                        .or_insert_with(Vec::new)
                        .push(result.device_id.clone());
                }
            }
        }

        output_groups
    }

    /// Format results as a table
    pub fn format_as_table(results: &BatchCommandResults) -> String {
        let mut table = String::new();

        // Add header
        table.push_str("+-----------------+-----------------+--------------------------------+--------+-----------+------------------+\n");
        table.push_str("| Device          | Type            | Command                        | Status | Duration  | Error            |\n");
        table.push_str("+-----------------+-----------------+--------------------------------+--------+-----------+------------------+\n");

        // Add rows
        for (_, device_results) in &results.results {
            for result in device_results {
                let status = match result.status {
                    CommandStatus::Success => "SUCCESS",
                    CommandStatus::Failed => "FAILED",
                    CommandStatus::Timeout => "TIMEOUT",
                    CommandStatus::Skipped => "SKIPPED",
                };

                // Truncate long values for table display
                let device = truncate(&result.device_id, 15);
                let device_type = truncate(&result.device_type, 15);
                let command = truncate(&result.command, 30);
                let error = match &result.error {
                    Some(err) => truncate(err, 16),
                    None => "".to_string(),
                };

                table.push_str(&format!(
                    "| {:<15} | {:<15} | {:<30} | {:<6} | {:<9} | {:<16} |\n",
                    device,
                    device_type,
                    command,
                    status,
                    format!("{} ms", result.duration_ms),
                    error
                ));
            }
        }

        // Add footer
        table.push_str("+-----------------+-----------------+--------------------------------+--------+-----------+------------------+\n");

        // Add summary
        table.push_str(&format!(
            "Summary: {} devices, {} commands ({} success, {} failed, {} timeout, {} skipped) in {} ms\n",
            results.device_count,
            results.command_count,
            results.success_count,
            results.failure_count,
            results.timeout_count,
            results.skipped_count,
            results.duration_ms
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
