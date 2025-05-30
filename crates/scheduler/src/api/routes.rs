use axum::{
    routing::{delete, get, post, put},
    Router,
};

use super::handlers::{
    cancel_scheduled_job, create_connection_profile, create_job, create_scheduled_job,
    delete_connection_profile, delete_job, get_connection_profile, get_job, get_job_logs,
    health_check, list_connection_profiles, list_jobs, list_scheduled_jobs,
    update_connection_profile, AppState,
};

pub fn create_api_routes() -> Router<AppState> {
    Router::new()
        // Health check
        .route("/health", get(health_check))
        // Job management routes
        .route("/jobs", post(create_job))
        .route("/jobs", get(list_jobs))
        .route("/jobs/:job_id", get(get_job))
        .route("/jobs/:job_id", delete(delete_job))
        .route("/jobs/:job_id/logs", get(get_job_logs))
        // Scheduled job routes
        .route("/scheduled-jobs", post(create_scheduled_job))
        .route("/scheduled-jobs", get(list_scheduled_jobs))
        .route("/scheduled-jobs/:job_id/cancel", post(cancel_scheduled_job))
        // SSH connection profile routes
        .route("/connections", post(create_connection_profile))
        .route("/connections", get(list_connection_profiles))
        .route("/connections/:profile_id", get(get_connection_profile))
        .route("/connections/:profile_id", put(update_connection_profile))
        .route(
            "/connections/:profile_id",
            delete(delete_connection_profile),
        )
}
