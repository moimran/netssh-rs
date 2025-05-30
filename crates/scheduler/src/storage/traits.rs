use async_trait::async_trait;
use uuid::Uuid;

use crate::error::StorageError;
use crate::jobs::types::{
    CommandResult, JobDetails, JobFilter, JobResult, JobSummary, SshConnectionProfile,
};

/// Trait defining the storage interface for job management
#[async_trait]
pub trait JobStorage: Send + Sync {
    /// Save a job result to storage
    async fn save_job_result(&self, result: &JobResult) -> Result<(), StorageError>;

    /// Get a job result by ID
    async fn get_job_result(&self, job_id: Uuid) -> Result<Option<JobResult>, StorageError>;

    /// List jobs with optional filtering
    async fn list_jobs(&self, filter: JobFilter) -> Result<Vec<JobSummary>, StorageError>;

    /// Delete a job and all associated data
    async fn delete_job(&self, job_id: Uuid) -> Result<(), StorageError>;

    /// Update job status
    async fn update_job_status(
        &self,
        job_id: Uuid,
        status: crate::jobs::types::JobStatus,
    ) -> Result<(), StorageError>;

    /// Save a scheduled job with scheduling information
    async fn save_scheduled_job(
        &self,
        job_id: Uuid,
        job_type: crate::jobs::types::JobType,
        payload: &str,
        scheduled_for: Option<chrono::DateTime<chrono::Utc>>,
        cron_expression: Option<&str>,
        next_run_at: Option<chrono::DateTime<chrono::Utc>>,
        max_retries: u32,
        description: Option<&str>,
    ) -> Result<(), StorageError>;

    /// Get job details including payload by job ID
    async fn get_job_details(&self, job_id: Uuid) -> Result<Option<JobDetails>, StorageError>;

    /// Save command results for a job
    async fn save_command_results(
        &self,
        job_id: Uuid,
        results: &[CommandResult],
    ) -> Result<(), StorageError>;

    /// Get command results for a job
    async fn get_command_results(&self, job_id: Uuid) -> Result<Vec<CommandResult>, StorageError>;

    /// Log a message for a job
    async fn log_job_message(
        &self,
        job_id: Uuid,
        level: &str,
        message: &str,
        context: Option<&str>,
    ) -> Result<(), StorageError>;

    /// Get logs for a job
    async fn get_job_logs(&self, job_id: Uuid) -> Result<Vec<JobLogEntry>, StorageError>;
}

/// Trait for managing SSH connection profiles
#[async_trait]
pub trait ConnectionStorage: Send + Sync {
    /// Save an SSH connection profile
    async fn save_connection_profile(
        &self,
        profile: &SshConnectionProfile,
    ) -> Result<(), StorageError>;

    /// Get an SSH connection profile by ID
    async fn get_connection_profile(
        &self,
        id: Uuid,
    ) -> Result<Option<SshConnectionProfile>, StorageError>;

    /// Get an SSH connection profile by name
    async fn get_connection_profile_by_name(
        &self,
        name: &str,
    ) -> Result<Option<SshConnectionProfile>, StorageError>;

    /// List all SSH connection profiles
    async fn list_connection_profiles(&self) -> Result<Vec<SshConnectionProfile>, StorageError>;

    /// Delete an SSH connection profile
    async fn delete_connection_profile(&self, id: Uuid) -> Result<(), StorageError>;

    /// Update an SSH connection profile
    async fn update_connection_profile(
        &self,
        profile: &SshConnectionProfile,
    ) -> Result<(), StorageError>;
}

/// Combined storage trait that includes both job and connection storage
#[async_trait]
pub trait Storage: JobStorage + ConnectionStorage + Send + Sync {
    /// Initialize the storage (run migrations, etc.)
    async fn initialize(&self) -> Result<(), StorageError>;

    /// Health check for the storage
    async fn health_check(&self) -> Result<(), StorageError>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JobLogEntry {
    pub id: i64,
    pub job_id: Uuid,
    pub level: String,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub context: Option<String>,
}
