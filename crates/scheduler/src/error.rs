use apalis_sql::sqlx;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchedulerError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("SSH connection error: {0}")]
    SshConnection(String),

    #[error("SSH command execution error: {0}")]
    SshExecution(String),

    #[error("Job not found: {0}")]
    JobNotFound(String),

    #[error("Invalid job configuration: {0}")]
    InvalidJobConfig(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Configuration(#[from] config::ConfigError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Apalis error: {0}")]
    Apalis(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, SchedulerError>;

// Storage-specific errors
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database connection error: {0}")]
    Connection(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
}

impl From<StorageError> for SchedulerError {
    fn from(err: StorageError) -> Self {
        match err {
            StorageError::Connection(e) => SchedulerError::Database(e),
            StorageError::Query(msg) => SchedulerError::Internal(msg),
            StorageError::Serialization(e) => SchedulerError::Serialization(e),
            StorageError::NotFound(msg) => SchedulerError::JobNotFound(msg),
            _ => SchedulerError::Internal(err.to_string()),
        }
    }
}
