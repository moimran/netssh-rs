use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

// Re-export FailureStrategy from config
pub use crate::config::FailureStrategy;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshJobPayload {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub connection: SshConnectionConfig,
    pub commands: Vec<String>,
    pub timeout: Option<Duration>,
    pub retry_count: Option<u32>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConnectionConfig {
    pub host: String,
    pub username: String,
    pub password: Option<String>,
    pub private_key: Option<String>,
    pub port: Option<u16>,
    pub device_type: String,
    pub timeout_seconds: Option<u64>,
    pub secret: Option<String>, // Enable password/secret for privileged mode
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobType {
    SshCommand,
    // Future job types can be added here
    // FileTransfer,
    // DatabaseBackup,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Retrying,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    pub job_id: Uuid,
    pub status: JobStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub command_results: Vec<CommandResult>,
    pub error: Option<String>,
    pub retry_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub id: Uuid,
    pub command: String,
    pub output: Option<String>,
    pub error: Option<String>,
    pub exit_code: Option<i32>,
    pub executed_at: DateTime<Utc>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSummary {
    pub id: Uuid,
    pub job_type: JobType,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub scheduled_for: Option<DateTime<Utc>>,
    pub cron_expression: Option<String>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub retry_count: u32,
    pub max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobDetails {
    pub id: Uuid,
    pub job_type: JobType,
    pub payload: serde_json::Value,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub scheduled_for: Option<DateTime<Utc>>,
    pub cron_expression: Option<String>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub retry_count: u32,
    pub max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobFilter {
    pub status: Option<JobStatus>,
    pub job_type: Option<JobType>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJobRequest {
    pub job_type: JobType,
    pub payload: serde_json::Value,
    pub scheduled_for: Option<DateTime<Utc>>,
    pub cron_expression: Option<String>,
    pub max_retries: Option<u32>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResponse {
    pub id: Uuid,
    pub status: JobStatus,
    pub message: String,
}

// SSH Connection Profile for reusable connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConnectionProfile {
    pub id: Uuid,
    pub name: String,
    pub config: SshConnectionConfig,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for JobFilter {
    fn default() -> Self {
        Self {
            status: None,
            job_type: None,
            created_after: None,
            created_before: None,
            limit: Some(50),
            offset: Some(0),
        }
    }
}

/// Scheduling configuration for jobs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScheduleType {
    /// Execute immediately (default behavior)
    Immediate,
    /// Execute once at a specific time
    OneTime { scheduled_for: DateTime<Utc> },
    /// Execute on a recurring schedule using cron expression
    Recurring {
        cron_expression: String,
        timezone: Option<String>, // e.g., "America/New_York", defaults to UTC
    },
}

impl Default for ScheduleType {
    fn default() -> Self {
        ScheduleType::Immediate
    }
}

/// Enhanced job creation request with scheduling support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScheduledJobRequest {
    pub job_type: JobType,
    pub payload: serde_json::Value,
    pub schedule: ScheduleType,
    pub max_retries: Option<u32>,
    pub description: Option<String>,
}

/// Job scheduling status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SchedulingStatus {
    /// Job is scheduled and waiting for execution time
    Scheduled,
    /// Job is due for execution
    Due,
    /// Recurring job that has been executed and rescheduled
    Rescheduled,
    /// One-time job that has been executed
    Executed,
    /// Job scheduling has been cancelled
    Cancelled,
}

/// Scheduled job information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledJob {
    pub id: Uuid,
    pub job_type: JobType,
    pub payload: serde_json::Value,
    pub schedule_type: ScheduleType,
    pub scheduling_status: SchedulingStatus,
    pub created_at: DateTime<Utc>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub last_run_at: Option<DateTime<Utc>>,
    pub run_count: u32,
    pub description: Option<String>,
}
