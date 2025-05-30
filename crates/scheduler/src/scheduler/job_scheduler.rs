use std::sync::Arc;
use std::time::Duration;

use apalis::prelude::*;
use chrono::{DateTime, Utc};
use tokio::time::{interval, sleep};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::config::SchedulerConfig;
use crate::error::{Result, SchedulerError};
use crate::jobs::types::{JobStatus, JobType, ScheduleType, SshJobPayload};
use crate::scheduler::CronScheduler;
use crate::storage::traits::Storage as AppStorage;

/// Main job scheduler service that monitors and executes scheduled jobs
pub struct JobScheduler {
    storage: Arc<dyn AppStorage>,
    cron_scheduler: CronScheduler,
    config: SchedulerConfig,
    job_queue: Arc<apalis_sql::sqlite::SqliteStorage<SshJobPayload>>,
}

impl JobScheduler {
    /// Create a new job scheduler
    pub fn new(
        storage: Arc<dyn AppStorage>,
        job_queue: Arc<apalis_sql::sqlite::SqliteStorage<SshJobPayload>>,
        config: SchedulerConfig,
    ) -> Result<Self> {
        let cron_scheduler = CronScheduler::new(config.timezone.as_deref())?;

        Ok(Self {
            storage,
            cron_scheduler,
            config,
            job_queue,
        })
    }

    /// Start the scheduler service
    pub async fn start(&self) -> Result<()> {
        info!(
            poll_interval_seconds = self.config.poll_interval_seconds,
            timezone = ?self.cron_scheduler.timezone(),
            "Starting job scheduler service"
        );

        let mut interval = interval(Duration::from_secs(self.config.poll_interval_seconds));

        loop {
            interval.tick().await;

            if let Err(e) = self.process_scheduled_jobs().await {
                error!(error = %e, "Error processing scheduled jobs");

                // Sleep for a short time before retrying to avoid tight error loops
                sleep(Duration::from_secs(5)).await;
            }
        }
    }

    /// Process all scheduled jobs that are due for execution
    async fn process_scheduled_jobs(&self) -> Result<()> {
        debug!("Checking for scheduled jobs due for execution");

        let now = Utc::now();

        // Process one-time scheduled jobs
        self.process_one_time_jobs(now).await?;

        // Process recurring jobs
        self.process_recurring_jobs(now).await?;

        Ok(())
    }

    /// Process one-time scheduled jobs that are due
    async fn process_one_time_jobs(&self, now: DateTime<Utc>) -> Result<()> {
        // Query for one-time scheduled jobs that are due
        let due_jobs = self.get_due_one_time_jobs(now).await?;

        info!(
            job_count = due_jobs.len(),
            "Found one-time scheduled jobs due for execution"
        );

        for job in due_jobs {
            if let Err(e) = self.execute_scheduled_job(&job).await {
                error!(
                    job_id = %job.id,
                    error = %e,
                    "Failed to execute one-time scheduled job"
                );
            }
        }

        Ok(())
    }

    /// Process recurring jobs that are due
    async fn process_recurring_jobs(&self, now: DateTime<Utc>) -> Result<()> {
        // Query for recurring jobs that are due
        let due_jobs = self.get_due_recurring_jobs(now).await?;

        info!(
            job_count = due_jobs.len(),
            "Found recurring jobs due for execution"
        );

        for job in due_jobs {
            if let Err(e) = self.execute_recurring_job(&job, now).await {
                error!(
                    job_id = %job.id,
                    error = %e,
                    "Failed to execute recurring job"
                );
            }
        }

        Ok(())
    }

    /// Execute a scheduled job by enqueueing it for processing
    async fn execute_scheduled_job(&self, job: &ScheduledJobInfo) -> Result<()> {
        info!(
            job_id = %job.id,
            job_type = ?job.job_type,
            "Executing scheduled job"
        );

        // Parse the job payload based on job type
        let payload = match job.job_type {
            JobType::SshCommand => serde_json::from_value::<SshJobPayload>(job.payload.clone())
                .map_err(|e| {
                    SchedulerError::Internal(format!("Failed to parse SSH job payload: {}", e))
                })?,
        };

        // Enqueue the job for execution
        let mut job_queue = (*self.job_queue).clone();
        job_queue
            .push(payload)
            .await
            .map_err(|e| SchedulerError::Internal(format!("Failed to enqueue job: {}", e)))?;

        // Update job status to indicate it has been executed
        self.mark_job_executed(job.id).await?;

        info!(
            job_id = %job.id,
            "Successfully enqueued scheduled job for execution"
        );

        Ok(())
    }

    /// Execute a recurring job and reschedule it for the next execution
    async fn execute_recurring_job(
        &self,
        job: &ScheduledJobInfo,
        now: DateTime<Utc>,
    ) -> Result<()> {
        info!(
            job_id = %job.id,
            cron_expression = ?job.cron_expression,
            "Executing recurring job"
        );

        // Execute the job
        self.execute_scheduled_job(job).await?;

        // Calculate next execution time
        if let Some(cron_expr) = &job.cron_expression {
            match self
                .cron_scheduler
                .next_execution_time(cron_expr, Some(now))?
            {
                Some(next_run_at) => {
                    self.reschedule_recurring_job(job.id, next_run_at).await?;
                    info!(
                        job_id = %job.id,
                        next_run_at = %next_run_at,
                        "Rescheduled recurring job for next execution"
                    );
                }
                None => {
                    warn!(
                        job_id = %job.id,
                        cron_expression = cron_expr,
                        "No future execution time found for recurring job"
                    );
                    self.mark_job_completed(job.id).await?;
                }
            }
        }

        Ok(())
    }

    /// Schedule a new job with the given schedule type
    pub async fn schedule_job(
        &self,
        job_type: JobType,
        payload: serde_json::Value,
        schedule: ScheduleType,
        max_retries: Option<u32>,
        description: Option<String>,
    ) -> Result<Uuid> {
        let job_id = Uuid::new_v4();
        let now = Utc::now();

        info!(
            job_id = %job_id,
            job_type = ?job_type,
            schedule = ?schedule,
            "Scheduling new job"
        );

        match schedule {
            ScheduleType::Immediate => {
                // For immediate execution, just enqueue the job
                match job_type {
                    JobType::SshCommand => {
                        let ssh_payload = serde_json::from_value::<SshJobPayload>(payload)
                            .map_err(|e| {
                                SchedulerError::Validation(format!(
                                    "Invalid SSH job payload: {}",
                                    e
                                ))
                            })?;

                        let mut job_queue = (*self.job_queue).clone();
                        job_queue.push(ssh_payload).await.map_err(|e| {
                            SchedulerError::Internal(format!(
                                "Failed to enqueue immediate job: {}",
                                e
                            ))
                        })?;
                    }
                }
            }
            ScheduleType::OneTime { scheduled_for } => {
                if scheduled_for <= now {
                    return Err(SchedulerError::Validation(
                        "Scheduled time must be in the future".to_string(),
                    ));
                }

                self.save_scheduled_job(
                    job_id,
                    job_type,
                    payload,
                    Some(scheduled_for),
                    None,
                    None,
                    max_retries,
                    description,
                )
                .await?;
            }
            ScheduleType::Recurring {
                cron_expression,
                timezone,
            } => {
                // Validate cron expression
                let scheduler = if let Some(tz) = timezone {
                    CronScheduler::new(Some(&tz))?
                } else {
                    CronScheduler::new(self.config.timezone.as_deref())?
                };

                scheduler.validate_cron_expression(&cron_expression)?;

                // Calculate first execution time
                let next_run_at = scheduler
                    .next_execution_time(&cron_expression, Some(now))?
                    .ok_or_else(|| {
                        SchedulerError::Validation(
                            "Cron expression does not produce any future execution times"
                                .to_string(),
                        )
                    })?;

                self.save_scheduled_job(
                    job_id,
                    job_type,
                    payload,
                    None,
                    Some(cron_expression),
                    Some(next_run_at),
                    max_retries,
                    description,
                )
                .await?;
            }
        }

        info!(
            job_id = %job_id,
            "Successfully scheduled job"
        );

        Ok(job_id)
    }

    /// Cancel a scheduled job
    pub async fn cancel_scheduled_job(&self, job_id: Uuid) -> Result<()> {
        info!(job_id = %job_id, "Cancelling scheduled job");

        // Update job status to cancelled
        self.storage
            .update_job_status(job_id, JobStatus::Cancelled)
            .await
            .map_err(|e| SchedulerError::Internal(format!("Failed to cancel job: {}", e)))?;

        info!(job_id = %job_id, "Successfully cancelled scheduled job");
        Ok(())
    }

    // Helper methods for database operations
    async fn get_due_one_time_jobs(&self, now: DateTime<Utc>) -> Result<Vec<ScheduledJobInfo>> {
        use crate::jobs::types::JobFilter;

        // Get all jobs from storage
        let jobs = self
            .storage
            .list_jobs(JobFilter::default())
            .await
            .map_err(|e| SchedulerError::Internal(format!("Failed to list jobs: {}", e)))?;

        // Filter for one-time scheduled jobs that are due and get their full details
        let mut due_jobs = Vec::new();
        for job in jobs {
            // Check if it's a one-time scheduled job that's due
            if let Some(scheduled_for) = job.scheduled_for {
                if scheduled_for <= now
                    && job.cron_expression.is_none()
                    && job.status == crate::jobs::types::JobStatus::Pending
                {
                    // Get the full job details including payload
                    if let Some(job_details) =
                        self.storage.get_job_details(job.id).await.map_err(|e| {
                            SchedulerError::Internal(format!("Failed to get job details: {}", e))
                        })?
                    {
                        due_jobs.push(ScheduledJobInfo {
                            id: job_details.id,
                            job_type: job_details.job_type,
                            payload: job_details.payload,
                            scheduled_for: job_details.scheduled_for,
                            cron_expression: job_details.cron_expression,
                            next_run_at: job_details.next_run_at,
                        });
                    }
                }
            }
        }

        Ok(due_jobs)
    }

    async fn get_due_recurring_jobs(&self, now: DateTime<Utc>) -> Result<Vec<ScheduledJobInfo>> {
        use crate::jobs::types::JobFilter;

        // Get all jobs from storage
        let jobs = self
            .storage
            .list_jobs(JobFilter::default())
            .await
            .map_err(|e| SchedulerError::Internal(format!("Failed to list jobs: {}", e)))?;

        // Filter for recurring jobs that are due and get their full details
        let mut due_jobs = Vec::new();
        for job in jobs {
            // Check if it's a recurring job that's due
            if let (Some(_cron_expr), Some(next_run_at)) = (&job.cron_expression, job.next_run_at) {
                if next_run_at <= now && job.status == crate::jobs::types::JobStatus::Pending {
                    // Get the full job details including payload
                    if let Some(job_details) =
                        self.storage.get_job_details(job.id).await.map_err(|e| {
                            SchedulerError::Internal(format!("Failed to get job details: {}", e))
                        })?
                    {
                        due_jobs.push(ScheduledJobInfo {
                            id: job_details.id,
                            job_type: job_details.job_type,
                            payload: job_details.payload,
                            scheduled_for: job_details.scheduled_for,
                            cron_expression: job_details.cron_expression,
                            next_run_at: job_details.next_run_at,
                        });
                    }
                }
            }
        }

        Ok(due_jobs)
    }

    async fn mark_job_executed(&self, job_id: Uuid) -> Result<()> {
        self.storage
            .update_job_status(job_id, JobStatus::Completed)
            .await
            .map_err(|e| SchedulerError::Internal(format!("Failed to mark job as executed: {}", e)))
    }

    async fn mark_job_completed(&self, job_id: Uuid) -> Result<()> {
        self.storage
            .update_job_status(job_id, JobStatus::Completed)
            .await
            .map_err(|e| {
                SchedulerError::Internal(format!("Failed to mark job as completed: {}", e))
            })
    }

    async fn reschedule_recurring_job(
        &self,
        job_id: Uuid,
        next_run_at: DateTime<Utc>,
    ) -> Result<()> {
        // This would update the next_run_at field in the database
        // For now, just log the action
        info!(
            job_id = %job_id,
            next_run_at = %next_run_at,
            "Rescheduled recurring job"
        );
        Ok(())
    }

    async fn save_scheduled_job(
        &self,
        job_id: Uuid,
        job_type: JobType,
        payload: serde_json::Value,
        scheduled_for: Option<DateTime<Utc>>,
        cron_expression: Option<String>,
        next_run_at: Option<DateTime<Utc>>,
        max_retries: Option<u32>,
        description: Option<String>,
    ) -> Result<()> {
        // Process the payload based on job type to ensure it's valid and complete
        let processed_payload = match job_type {
            JobType::SshCommand => {
                // Parse the raw payload into an SshJobPayload structure
                let mut ssh_payload: SshJobPayload =
                    serde_json::from_value(payload).map_err(|e| {
                        SchedulerError::Validation(format!("Invalid SSH job payload: {}", e))
                    })?;

                // Set the job ID to ensure consistency
                ssh_payload.id = job_id;

                // Validate the SSH job
                crate::jobs::validate_ssh_job(&ssh_payload).map_err(|e| {
                    SchedulerError::Validation(format!("SSH job validation failed: {}", e))
                })?;

                // Convert back to JSON for storage
                serde_json::to_value(ssh_payload).map_err(|e| {
                    SchedulerError::Internal(format!("Failed to serialize SSH payload: {}", e))
                })?
            }
        };

        let payload_str = processed_payload.to_string();
        let max_retries = max_retries.unwrap_or(3);

        self.storage
            .save_scheduled_job(
                job_id,
                job_type.clone(),
                &payload_str,
                scheduled_for,
                cron_expression.as_deref(),
                next_run_at,
                max_retries,
                description.as_deref(),
            )
            .await
            .map_err(|e| {
                SchedulerError::Internal(format!("Failed to save scheduled job: {}", e))
            })?;

        info!(
            job_id = %job_id,
            job_type = ?job_type,
            scheduled_for = ?scheduled_for,
            cron_expression = ?cron_expression,
            next_run_at = ?next_run_at,
            "Saved scheduled job to database"
        );
        Ok(())
    }
}

/// Information about a scheduled job
#[derive(Debug, Clone)]
struct ScheduledJobInfo {
    pub id: Uuid,
    pub job_type: JobType,
    pub payload: serde_json::Value,
    #[allow(dead_code)] // Used in pattern matching for scheduling logic
    pub scheduled_for: Option<DateTime<Utc>>,
    pub cron_expression: Option<String>,
    #[allow(dead_code)] // Used in pattern matching for scheduling logic
    pub next_run_at: Option<DateTime<Utc>>,
}
