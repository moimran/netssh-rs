use scheduler::{
    config::LoggingConfig,
    logging,
    storage::{SqliteStorage, Storage},
};

#[tokio::test]
async fn test_logging_initialization() {
    // Test console logging
    let config = LoggingConfig {
        level: "info".to_string(),
        file: None,
        format: Some("text".to_string()),
        rotation: None,
    };

    // This should not panic
    let result = logging::init_logging(&config);
    assert!(
        result.is_ok(),
        "Failed to initialize console logging: {:?}",
        result
    );
}

#[tokio::test]
async fn test_structured_logging_with_storage() {
    // Skip logging initialization since it may already be set

    // Create storage which should generate logs
    let storage = SqliteStorage::new("sqlite::memory:").await;
    assert!(storage.is_ok(), "Failed to create in-memory SQLite storage");

    // Test health check which should generate logs
    let storage = storage.unwrap();
    let health_result = storage.health_check().await;
    assert!(health_result.is_ok(), "Health check failed");
}

#[tokio::test]
async fn test_ssh_job_validation_logging() {
    use scheduler::jobs::types::{SshConnectionConfig, SshJobPayload};
    use std::time::Duration;
    use uuid::Uuid;

    // Skip logging initialization since it may already be set

    // Create a valid SSH job
    let ssh_job = SshJobPayload {
        id: Uuid::new_v4(),
        connection: SshConnectionConfig {
            host: "192.168.1.1".to_string(),
            username: "admin".to_string(),
            password: Some("password".to_string()),
            private_key: None,
            port: Some(22),
            device_type: "cisco_ios".to_string(),
            timeout_seconds: Some(60),
            secret: None, // No enable secret for this test
        },
        commands: vec!["show version".to_string()],
        timeout: Some(Duration::from_secs(300)),
        retry_count: Some(0),
        description: Some("Test SSH job".to_string()),
    };

    // Validate the job (this should generate logs)
    let result = scheduler::jobs::validate_ssh_job(&ssh_job);
    assert!(result.is_ok(), "SSH job validation failed: {:?}", result);

    // Test invalid job (should generate warning logs)
    let mut invalid_job = ssh_job;
    invalid_job.connection.host = "".to_string(); // Invalid empty host

    let result = scheduler::jobs::validate_ssh_job(&invalid_job);
    assert!(
        result.is_err(),
        "Expected validation to fail for invalid job"
    );
}
