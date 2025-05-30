use std::sync::Arc;

use apalis::prelude::Storage as ApalisStorage;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use chrono::Utc;
use serde_json::Value;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::jobs::types::{
    CreateJobRequest, CreateScheduledJobRequest, JobFilter, JobResponse, JobStatus, JobType,
    ScheduleType, SshConnectionProfile, SshJobPayload,
};
use crate::storage::traits::Storage;

pub type AppState = Arc<dyn Storage>;

/// Health check endpoint
pub async fn health_check(State(storage): State<AppState>) -> Result<Json<Value>, StatusCode> {
    info!("Health check requested");

    match storage.health_check().await {
        Ok(()) => {
            info!("Health check passed");
            Ok(Json(serde_json::json!({
                "status": "healthy",
                "timestamp": Utc::now()
            })))
        }
        Err(e) => {
            error!(error = %e, "Health check failed");
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

/// Create a new job
pub async fn create_job(
    State(storage): State<AppState>,
    Extension(mut job_queue): Extension<apalis_sql::sqlite::SqliteStorage<SshJobPayload>>,
    Json(request): Json<CreateJobRequest>,
) -> Result<Json<JobResponse>, StatusCode> {
    info!(
        job_type = ?request.job_type,
        "Creating new job"
    );

    match request.job_type {
        JobType::SshCommand => {
            // Parse SSH job payload
            let ssh_payload: SshJobPayload =
                serde_json::from_value(request.payload).map_err(|e| {
                    warn!(
                        error = %e,
                        "Failed to parse SSH job payload"
                    );
                    StatusCode::BAD_REQUEST
                })?;

            // Ensure we have a valid job ID (serde default will generate one if not provided)
            let job_id = ssh_payload.id;

            info!(
                job_id = %job_id,
                "Processing SSH job with ID: {}",
                job_id
            );

            // Validate the job
            crate::jobs::validate_ssh_job(&ssh_payload).map_err(|e| {
                warn!(
                    job_id = %job_id,
                    error = %e,
                    "SSH job validation failed"
                );
                StatusCode::BAD_REQUEST
            })?;

            // Create a job result record to store in database
            let job_result = crate::jobs::types::JobResult {
                job_id,
                status: JobStatus::Pending,
                started_at: None,
                completed_at: None,
                command_results: Vec::new(),
                error: None,
                retry_count: 0,
            };

            // Store the job in the database
            if let Err(e) = storage.save_job_result(&job_result).await {
                error!(
                    job_id = %job_id,
                    error = %e,
                    "Failed to save job to database"
                );
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }

            // Log the job creation with payload details for debugging
            info!(
                job_id = %job_id,
                payload_size = serde_json::to_string(&ssh_payload).map(|s| s.len()).unwrap_or(0),
                "Job payload processed and stored"
            );

            info!(
                job_id = %job_id,
                host = %ssh_payload.connection.host,
                command_count = ssh_payload.commands.len(),
                "SSH job created and stored successfully"
            );

            // Schedule the job with Apalis for execution
            info!(
                job_id = %job_id,
                "Attempting to push job to Apalis queue..."
            );

            match job_queue.push(ssh_payload).await {
                Ok(_) => {
                    info!(
                        job_id = %job_id,
                        "Job successfully scheduled with Apalis worker"
                    );

                    Ok(Json(JobResponse {
                        id: job_id,
                        status: JobStatus::Pending,
                        message: "Job created and stored successfully".to_string(),
                    }))
                }
                Err(e) => {
                    error!(
                        job_id = %job_id,
                        error = %e,
                        "Failed to schedule job with Apalis worker"
                    );

                    // Job is stored but not scheduled - return error
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
    }
}

/// Get job status and results
pub async fn get_job(
    State(storage): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    info!(job_id = %job_id, "Getting job details");

    match storage.get_job_result(job_id).await {
        Ok(Some(result)) => {
            let command_results = storage.get_command_results(job_id).await.map_err(|e| {
                error!(job_id = %job_id, error = %e, "Failed to get command results");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            info!(job_id = %job_id, "Job details retrieved successfully");
            Ok(Json(serde_json::json!({
                "job": result,
                "command_results": command_results
            })))
        }
        Ok(None) => {
            warn!(job_id = %job_id, "Job not found");
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            error!(job_id = %job_id, error = %e, "Failed to get job");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// List jobs with optional filtering
pub async fn list_jobs(
    State(storage): State<AppState>,
    Query(filter): Query<JobFilter>,
) -> Result<Json<Value>, StatusCode> {
    info!(
        status = ?filter.status,
        limit = ?filter.limit,
        "Listing jobs"
    );

    match storage.list_jobs(filter).await {
        Ok(jobs) => {
            info!(job_count = jobs.len(), "Jobs retrieved successfully");
            Ok(Json(serde_json::json!({
                "jobs": jobs,
                "count": jobs.len()
            })))
        }
        Err(e) => {
            error!(error = %e, "Failed to list jobs");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete a job
pub async fn delete_job(
    State(storage): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    info!(job_id = %job_id, "Deleting job");

    match storage.delete_job(job_id).await {
        Ok(_) => {
            info!(job_id = %job_id, "Job deleted successfully");
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            error!(job_id = %job_id, error = %e, "Failed to delete job");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get job logs
pub async fn get_job_logs(
    State(storage): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    info!(job_id = %job_id, "Getting job logs");

    match storage.get_job_logs(job_id).await {
        Ok(logs) => {
            info!(job_id = %job_id, log_count = logs.len(), "Job logs retrieved successfully");
            Ok(Json(serde_json::json!({
                "logs": logs
            })))
        }
        Err(e) => {
            error!(job_id = %job_id, error = %e, "Failed to get job logs");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// SSH Connection Profile endpoints

/// Create SSH connection profile
pub async fn create_connection_profile(
    State(storage): State<AppState>,
    Json(mut profile): Json<SshConnectionProfile>,
) -> Result<Json<SshConnectionProfile>, StatusCode> {
    profile.id = Uuid::new_v4();
    profile.created_at = Utc::now();
    profile.updated_at = Utc::now();

    info!(
        profile_id = %profile.id,
        profile_name = %profile.name,
        "Creating SSH connection profile"
    );

    match storage.save_connection_profile(&profile).await {
        Ok(()) => {
            info!(
                profile_id = %profile.id,
                profile_name = %profile.name,
                "SSH connection profile created successfully"
            );
            Ok(Json(profile))
        }
        Err(e) => {
            error!(
                profile_id = %profile.id,
                profile_name = %profile.name,
                error = %e,
                "Failed to create SSH connection profile"
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get SSH connection profile
pub async fn get_connection_profile(
    State(storage): State<AppState>,
    Path(profile_id): Path<Uuid>,
) -> Result<Json<SshConnectionProfile>, StatusCode> {
    info!(profile_id = %profile_id, "Getting SSH connection profile");

    match storage.get_connection_profile(profile_id).await {
        Ok(Some(profile)) => {
            info!(
                profile_id = %profile_id,
                profile_name = %profile.name,
                "SSH connection profile retrieved successfully"
            );
            Ok(Json(profile))
        }
        Ok(None) => {
            warn!(profile_id = %profile_id, "SSH connection profile not found");
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            error!(
                profile_id = %profile_id,
                error = %e,
                "Failed to get SSH connection profile"
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// List SSH connection profiles
pub async fn list_connection_profiles(
    State(storage): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    info!("Listing SSH connection profiles");

    match storage.list_connection_profiles().await {
        Ok(profiles) => {
            info!(
                profile_count = profiles.len(),
                "SSH connection profiles retrieved successfully"
            );
            Ok(Json(serde_json::json!({
                "profiles": profiles,
                "count": profiles.len()
            })))
        }
        Err(e) => {
            error!(error = %e, "Failed to list SSH connection profiles");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Update SSH connection profile
pub async fn update_connection_profile(
    State(storage): State<AppState>,
    Path(profile_id): Path<Uuid>,
    Json(mut profile): Json<SshConnectionProfile>,
) -> Result<Json<SshConnectionProfile>, StatusCode> {
    profile.id = profile_id;
    profile.updated_at = Utc::now();

    info!(
        profile_id = %profile_id,
        profile_name = %profile.name,
        "Updating SSH connection profile"
    );

    match storage.update_connection_profile(&profile).await {
        Ok(()) => {
            info!(
                profile_id = %profile_id,
                profile_name = %profile.name,
                "SSH connection profile updated successfully"
            );
            Ok(Json(profile))
        }
        Err(e) => {
            error!(
                profile_id = %profile_id,
                profile_name = %profile.name,
                error = %e,
                "Failed to update SSH connection profile"
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete SSH connection profile
pub async fn delete_connection_profile(
    State(storage): State<AppState>,
    Path(profile_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    info!(profile_id = %profile_id, "Deleting SSH connection profile");

    match storage.delete_connection_profile(profile_id).await {
        Ok(()) => {
            info!(profile_id = %profile_id, "SSH connection profile deleted successfully");
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            error!(
                profile_id = %profile_id,
                error = %e,
                "Failed to delete SSH connection profile"
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create a new scheduled job with enhanced scheduling support
pub async fn create_scheduled_job(
    State(storage): State<AppState>,
    Extension(job_queue): Extension<apalis_sql::sqlite::SqliteStorage<SshJobPayload>>,
    Extension(job_scheduler): Extension<std::sync::Arc<crate::scheduler::JobScheduler>>,
    Json(request): Json<CreateScheduledJobRequest>,
) -> Result<Json<JobResponse>, StatusCode> {
    info!(
        job_type = ?request.job_type,
        schedule = ?request.schedule,
        "Creating new scheduled job"
    );

    match request.schedule {
        ScheduleType::Immediate => {
            // For immediate execution, use the existing create_job logic
            let legacy_request = CreateJobRequest {
                job_type: request.job_type,
                payload: request.payload,
                scheduled_for: None,
                cron_expression: None,
                max_retries: request.max_retries,
                description: request.description,
            };
            create_job(State(storage), Extension(job_queue), Json(legacy_request)).await
        }
        ScheduleType::OneTime { scheduled_for } => {
            info!(
                scheduled_for = %scheduled_for,
                "Scheduling one-time job"
            );

            // Use the JobScheduler to schedule the job
            match job_scheduler
                .schedule_job(
                    request.job_type,
                    request.payload,
                    request.schedule,
                    request.max_retries,
                    request.description,
                )
                .await
            {
                Ok(job_id) => {
                    info!(
                        job_id = %job_id,
                        scheduled_for = %scheduled_for,
                        "One-time job scheduled successfully"
                    );

                    Ok(Json(JobResponse {
                        id: job_id,
                        status: JobStatus::Pending,
                        message: format!("Job scheduled for execution at {}", scheduled_for),
                    }))
                }
                Err(e) => {
                    error!(
                        error = %e,
                        scheduled_for = %scheduled_for,
                        "Failed to schedule one-time job"
                    );
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        ScheduleType::Recurring {
            cron_expression,
            timezone,
        } => {
            let job_id = Uuid::new_v4();

            // TODO: Validate cron expression using CronScheduler
            // TODO: Calculate next execution time
            // TODO: Save recurring job to database

            info!(
                job_id = %job_id,
                cron_expression = %cron_expression,
                timezone = ?timezone,
                "Scheduling recurring job"
            );

            Ok(Json(JobResponse {
                id: job_id,
                status: JobStatus::Pending,
                message: format!(
                    "Recurring job scheduled with cron expression: {}",
                    cron_expression
                ),
            }))
        }
    }
}

/// List scheduled jobs (jobs with future execution times)
pub async fn list_scheduled_jobs(
    State(storage): State<AppState>,
    Query(filter): Query<JobFilter>,
) -> Result<Json<Value>, StatusCode> {
    info!("Listing scheduled jobs");

    // Modify filter to only include scheduled jobs
    // This would need to be implemented in the storage layer
    match storage.list_jobs(filter).await {
        Ok(jobs) => {
            // Filter for jobs that have scheduling information
            let scheduled_jobs: Vec<_> = jobs
                .into_iter()
                .filter(|job| job.scheduled_for.is_some() || job.cron_expression.is_some())
                .collect();

            info!(
                scheduled_job_count = scheduled_jobs.len(),
                "Scheduled jobs retrieved successfully"
            );

            Ok(Json(serde_json::json!({
                "scheduled_jobs": scheduled_jobs,
                "count": scheduled_jobs.len()
            })))
        }
        Err(e) => {
            error!(error = %e, "Failed to list scheduled jobs");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Cancel a scheduled job
pub async fn cancel_scheduled_job(
    State(storage): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    info!(job_id = %job_id, "Cancelling scheduled job");

    match storage
        .update_job_status(job_id, JobStatus::Cancelled)
        .await
    {
        Ok(()) => {
            info!(job_id = %job_id, "Scheduled job cancelled successfully");
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            error!(
                job_id = %job_id,
                error = %e,
                "Failed to cancel scheduled job"
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
