use async_trait::async_trait;
use sqlx::{Row, SqlitePool, sqlite::SqliteConnectOptions};
use std::str::FromStr;
use std::path::Path;
use tracing::{error, info};
use uuid::Uuid;

use crate::error::StorageError;
use crate::jobs::types::{
    CommandResult, JobDetails, JobFilter, JobResult, JobStatus, JobSummary, JobType,
    SshConnectionProfile,
};
use crate::storage::traits::{ConnectionStorage, JobLogEntry, JobStorage, Storage};

pub struct SqliteStorage {
    pool: SqlitePool,
}

impl SqliteStorage {
    pub async fn new(database_url: &str) -> Result<Self, StorageError> {
        info!(database_url = %database_url, "Initializing SQLite database");

        // Extract the database file path from the URL
        let db_path = if database_url.starts_with("sqlite:") {
            &database_url[7..] // Remove "sqlite:" prefix
        } else {
            database_url
        };

        // Create the directory if it doesn't exist
        if let Some(parent_dir) = Path::new(db_path).parent() {
            if !parent_dir.exists() {
                info!(directory = %parent_dir.display(), "Creating database directory");
                std::fs::create_dir_all(parent_dir).map_err(|e| {
                    error!(directory = %parent_dir.display(), error = %e, "Failed to create database directory");
                    StorageError::Migration(format!("Failed to create database directory: {}", e))
                })?;
            }
        }

        // Use SqliteConnectOptions to create the database if it doesn't exist
        let connect_options = SqliteConnectOptions::from_str(database_url)
            .map_err(|e| {
                error!(database_url = %database_url, error = %e, "Invalid database URL");
                StorageError::Migration(format!("Invalid database URL: {}", e))
            })?
            .create_if_missing(true);

        info!(database_url = %database_url, "Connecting to SQLite database");
        let pool = SqlitePool::connect_with(connect_options).await.map_err(|e| {
            error!(database_url = %database_url, error = %e, "Failed to connect to SQLite database");
            e
        })?;

        info!("Creating database tables and indexes");
        // Create tables manually to avoid migration macro issues
        Self::create_tables(&pool).await?;

        info!("SQLite storage initialized successfully");
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    async fn create_tables(pool: &SqlitePool) -> Result<(), StorageError> {
        info!("Creating jobs table");
        // Create jobs table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS jobs (
                id TEXT PRIMARY KEY,
                job_type TEXT NOT NULL,
                payload TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                started_at DATETIME,
                completed_at DATETIME,
                scheduled_for DATETIME,
                cron_expression TEXT,
                next_run_at DATETIME,
                retry_count INTEGER DEFAULT 0,
                max_retries INTEGER DEFAULT 3,
                error_message TEXT,
                worker_id TEXT
            )
            "#,
        )
        .execute(pool)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to create jobs table");
            StorageError::Migration(format!("Failed to create jobs table: {}", e))
        })?;

        info!("Creating job_results table");
        // Create job results table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS job_results (
                id TEXT PRIMARY KEY,
                job_id TEXT NOT NULL,
                command TEXT NOT NULL,
                output TEXT,
                error TEXT,
                exit_code INTEGER,
                executed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                duration_ms INTEGER,
                FOREIGN KEY (job_id) REFERENCES jobs (id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(pool)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to create job_results table");
            StorageError::Migration(format!("Failed to create job_results table: {}", e))
        })?;

        info!("Creating job_logs table");
        // Create job logs table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS job_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                job_id TEXT NOT NULL,
                level TEXT NOT NULL,
                message TEXT NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                context TEXT,
                FOREIGN KEY (job_id) REFERENCES jobs (id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(pool)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to create job_logs table");
            StorageError::Migration(format!("Failed to create job_logs table: {}", e))
        })?;

        info!("Creating ssh_connections table");
        // Create SSH connections table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS ssh_connections (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                host TEXT NOT NULL,
                port INTEGER NOT NULL DEFAULT 22,
                username TEXT NOT NULL,
                password TEXT,
                private_key_path TEXT,
                device_type TEXT,
                timeout_seconds INTEGER DEFAULT 30,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(pool)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to create ssh_connections table");
            StorageError::Migration(format!("Failed to create ssh_connections table: {}", e))
        })?;

        // Create indexes for better performance
        info!("Creating database indexes");
        Self::create_indexes(pool).await?;

        info!("All database tables and indexes created successfully");
        Ok(())
    }

    async fn create_indexes(pool: &SqlitePool) -> Result<(), StorageError> {
        let indexes = vec![
            ("idx_jobs_status", "CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status)"),
            ("idx_jobs_created_at", "CREATE INDEX IF NOT EXISTS idx_jobs_created_at ON jobs(created_at)"),
            ("idx_jobs_scheduled_for", "CREATE INDEX IF NOT EXISTS idx_jobs_scheduled_for ON jobs(scheduled_for)"),
            ("idx_jobs_next_run_at", "CREATE INDEX IF NOT EXISTS idx_jobs_next_run_at ON jobs(next_run_at)"),
            ("idx_jobs_job_type", "CREATE INDEX IF NOT EXISTS idx_jobs_job_type ON jobs(job_type)"),
            ("idx_jobs_cron_expression", "CREATE INDEX IF NOT EXISTS idx_jobs_cron_expression ON jobs(cron_expression)"),
            ("idx_jobs_scheduler_query", "CREATE INDEX IF NOT EXISTS idx_jobs_scheduler_query ON jobs(status, next_run_at, scheduled_for)"),
            ("idx_job_results_job_id", "CREATE INDEX IF NOT EXISTS idx_job_results_job_id ON job_results(job_id)"),
            ("idx_job_results_executed_at", "CREATE INDEX IF NOT EXISTS idx_job_results_executed_at ON job_results(executed_at)"),
            ("idx_job_logs_job_id", "CREATE INDEX IF NOT EXISTS idx_job_logs_job_id ON job_logs(job_id)"),
            ("idx_job_logs_timestamp", "CREATE INDEX IF NOT EXISTS idx_job_logs_timestamp ON job_logs(timestamp)"),
            ("idx_ssh_connections_name", "CREATE INDEX IF NOT EXISTS idx_ssh_connections_name ON ssh_connections(name)"),
        ];

        for (index_name, sql) in indexes {
            sqlx::query(sql)
                .execute(pool)
                .await
                .map_err(|e| {
                    error!(index = %index_name, error = %e, "Failed to create index");
                    StorageError::Migration(format!("Failed to create index {}: {}", index_name, e))
                })?;
        }

        Ok(())
    }
}

#[async_trait]
impl JobStorage for SqliteStorage {
    async fn save_job_result(&self, result: &JobResult) -> Result<(), StorageError> {
        info!(
            job_id = %result.job_id,
            status = ?result.status,
            retry_count = result.retry_count,
            "Saving job result"
        );

        let status_str = match result.status {
            JobStatus::Pending => "pending",
            JobStatus::Running => "running",
            JobStatus::Completed => "completed",
            JobStatus::Failed => "failed",
            JobStatus::Cancelled => "cancelled",
            JobStatus::Retrying => "retrying",
        };

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO jobs (
                id, job_type, payload, status, started_at, completed_at, retry_count, error_message,
                scheduled_for, cron_expression, next_run_at, max_retries
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(result.job_id.to_string())
        .bind("ssh_command")
        .bind("{}")
        .bind(status_str)
        .bind(result.started_at)
        .bind(result.completed_at)
        .bind(result.retry_count as i64)
        .bind(&result.error)
        .bind(None::<chrono::DateTime<chrono::Utc>>) // scheduled_for
        .bind(None::<String>) // cron_expression
        .bind(None::<chrono::DateTime<chrono::Utc>>) // next_run_at
        .bind(3i64) // max_retries default
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!(
                job_id = %result.job_id,
                error = %e,
                "Failed to save job result"
            );
            e
        })?;

        info!(job_id = %result.job_id, "Job result saved successfully");
        Ok(())
    }

    async fn get_job_result(&self, job_id: Uuid) -> Result<Option<JobResult>, StorageError> {
        let job_row = sqlx::query("SELECT * FROM jobs WHERE id = ?")
            .bind(job_id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = job_row {
            let status_str: String = row.get("status");
            let status = match status_str.as_str() {
                "pending" => JobStatus::Pending,
                "running" => JobStatus::Running,
                "completed" => JobStatus::Completed,
                "failed" => JobStatus::Failed,
                "cancelled" => JobStatus::Cancelled,
                "retrying" => JobStatus::Retrying,
                _ => JobStatus::Pending,
            };

            Ok(Some(JobResult {
                job_id,
                status,
                started_at: row.get("started_at"),
                completed_at: row.get("completed_at"),
                command_results: Vec::new(), // Simplified for now
                error: row.get("error_message"),
                retry_count: row.get::<i64, _>("retry_count") as u32,
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_jobs(&self, filter: JobFilter) -> Result<Vec<JobSummary>, StorageError> {
        info!(
            status_filter = ?filter.status,
            limit = ?filter.limit,
            "Listing jobs with filter"
        );

        let rows = sqlx::query("SELECT * FROM jobs ORDER BY created_at DESC LIMIT 50")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to execute list jobs query");
                e
            })?;

        let mut summaries = Vec::new();
        for row in rows {
            let id_str: String = row.get("id");
            let job_id = Uuid::from_str(&id_str)
                .map_err(|e| StorageError::Query(format!("Invalid UUID: {}", e)))?;

            let status_str: String = row.get("status");
            let status = match status_str.as_str() {
                "pending" => JobStatus::Pending,
                "running" => JobStatus::Running,
                "completed" => JobStatus::Completed,
                "failed" => JobStatus::Failed,
                "cancelled" => JobStatus::Cancelled,
                "retrying" => JobStatus::Retrying,
                _ => JobStatus::Pending,
            };

            summaries.push(JobSummary {
                id: job_id,
                job_type: JobType::SshCommand,
                status,
                created_at: row.get("created_at"),
                started_at: row.get("started_at"),
                completed_at: row.get("completed_at"),
                scheduled_for: row.get("scheduled_for"),
                cron_expression: row.get("cron_expression"),
                next_run_at: row.get("next_run_at"),
                description: None,
                retry_count: row.get::<i64, _>("retry_count") as u32,
                max_retries: row.get::<i64, _>("max_retries") as u32,
            });
        }

        info!(
            job_count = summaries.len(),
            "Successfully retrieved job summaries"
        );
        Ok(summaries)
    }

    async fn delete_job(&self, job_id: Uuid) -> Result<(), StorageError> {
        info!(job_id = %job_id, "Deleting job from storage");

        let result = sqlx::query("DELETE FROM jobs WHERE id = ?")
            .bind(job_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| {
                error!(job_id = %job_id, error = %e, "Failed to delete job");
                e
            })?;

        info!(
            job_id = %job_id,
            rows_affected = result.rows_affected(),
            "Job deleted successfully"
        );
        Ok(())
    }

    async fn update_job_status(&self, job_id: Uuid, status: JobStatus) -> Result<(), StorageError> {
        let status_str = match status {
            JobStatus::Pending => "pending",
            JobStatus::Running => "running",
            JobStatus::Completed => "completed",
            JobStatus::Failed => "failed",
            JobStatus::Cancelled => "cancelled",
            JobStatus::Retrying => "retrying",
        };

        sqlx::query("UPDATE jobs SET status = ? WHERE id = ?")
            .bind(status_str)
            .bind(job_id.to_string())
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn save_scheduled_job(
        &self,
        job_id: Uuid,
        job_type: JobType,
        payload: &str,
        scheduled_for: Option<chrono::DateTime<chrono::Utc>>,
        cron_expression: Option<&str>,
        next_run_at: Option<chrono::DateTime<chrono::Utc>>,
        max_retries: u32,
        _description: Option<&str>,
    ) -> Result<(), StorageError> {
        let job_type_str = match job_type {
            JobType::SshCommand => "ssh_command",
        };

        sqlx::query(
            r#"
            INSERT INTO jobs (
                id, job_type, payload, status, scheduled_for, cron_expression,
                next_run_at, max_retries, retry_count, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            "#,
        )
        .bind(job_id.to_string())
        .bind(job_type_str)
        .bind(payload)
        .bind("pending")
        .bind(scheduled_for)
        .bind(cron_expression)
        .bind(next_run_at)
        .bind(max_retries as i64)
        .bind(0i64) // retry_count
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to save scheduled job: {}", e);
            StorageError::Query(e.to_string())
        })?;

        info!(
            job_id = %job_id,
            job_type = %job_type_str,
            scheduled_for = ?scheduled_for,
            cron_expression = ?cron_expression,
            next_run_at = ?next_run_at,
            max_retries = %max_retries,
            "Scheduled job saved to database"
        );
        Ok(())
    }

    async fn get_job_details(&self, job_id: Uuid) -> Result<Option<JobDetails>, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT id, job_type, payload, status, created_at, started_at, completed_at,
                   scheduled_for, cron_expression, next_run_at, retry_count, max_retries
            FROM jobs WHERE id = ?
            "#,
        )
        .bind(job_id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let payload_str: String = row.get("payload");
            let payload: serde_json::Value = serde_json::from_str(&payload_str)
                .map_err(|e| StorageError::Query(format!("Failed to parse payload JSON: {}", e)))?;

            let status_str: String = row.get("status");
            let status = match status_str.as_str() {
                "pending" => JobStatus::Pending,
                "running" => JobStatus::Running,
                "completed" => JobStatus::Completed,
                "failed" => JobStatus::Failed,
                "cancelled" => JobStatus::Cancelled,
                "retrying" => JobStatus::Retrying,
                _ => JobStatus::Pending,
            };

            Ok(Some(JobDetails {
                id: job_id,
                job_type: JobType::SshCommand, // For now, only SSH jobs are supported
                payload,
                status,
                created_at: row.get("created_at"),
                started_at: row.get("started_at"),
                completed_at: row.get("completed_at"),
                scheduled_for: row.get("scheduled_for"),
                cron_expression: row.get("cron_expression"),
                next_run_at: row.get("next_run_at"),
                description: None, // TODO: Add description field to database
                retry_count: row.get::<i64, _>("retry_count") as u32,
                max_retries: row.get::<i64, _>("max_retries") as u32,
            }))
        } else {
            Ok(None)
        }
    }

    async fn save_command_results(
        &self,
        _job_id: Uuid,
        _results: &[CommandResult],
    ) -> Result<(), StorageError> {
        // Simplified implementation - just return Ok for now
        Ok(())
    }

    async fn get_command_results(&self, _job_id: Uuid) -> Result<Vec<CommandResult>, StorageError> {
        // Simplified implementation - return empty vec for now
        Ok(Vec::new())
    }

    async fn log_job_message(
        &self,
        _job_id: Uuid,
        _level: &str,
        _message: &str,
        _context: Option<&str>,
    ) -> Result<(), StorageError> {
        // Simplified implementation - just return Ok for now
        Ok(())
    }

    async fn get_job_logs(&self, _job_id: Uuid) -> Result<Vec<JobLogEntry>, StorageError> {
        // Simplified implementation - return empty vec for now
        Ok(Vec::new())
    }
}

#[async_trait]
impl ConnectionStorage for SqliteStorage {
    async fn save_connection_profile(
        &self,
        _profile: &SshConnectionProfile,
    ) -> Result<(), StorageError> {
        // Simplified implementation - just return Ok for now
        Ok(())
    }

    async fn get_connection_profile(
        &self,
        _id: Uuid,
    ) -> Result<Option<SshConnectionProfile>, StorageError> {
        // Simplified implementation - return None for now
        Ok(None)
    }

    async fn get_connection_profile_by_name(
        &self,
        _name: &str,
    ) -> Result<Option<SshConnectionProfile>, StorageError> {
        // Simplified implementation - return None for now
        Ok(None)
    }

    async fn list_connection_profiles(&self) -> Result<Vec<SshConnectionProfile>, StorageError> {
        // Simplified implementation - return empty vec for now
        Ok(Vec::new())
    }

    async fn delete_connection_profile(&self, _id: Uuid) -> Result<(), StorageError> {
        // Simplified implementation - just return Ok for now
        Ok(())
    }

    async fn update_connection_profile(
        &self,
        _profile: &SshConnectionProfile,
    ) -> Result<(), StorageError> {
        // Simplified implementation - just return Ok for now
        Ok(())
    }
}

#[async_trait]
impl Storage for SqliteStorage {
    async fn initialize(&self) -> Result<(), StorageError> {
        Ok(())
    }

    async fn health_check(&self) -> Result<(), StorageError> {
        info!("Performing database health check");

        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                error!(error = %e, "Database health check failed");
                e
            })?;

        info!("Database health check passed");
        Ok(())
    }
}
