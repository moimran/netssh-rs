use std::time::Duration;
use uuid::Uuid;

use scheduler::{
    jobs::types::{SshConnectionConfig, SshJobPayload},
    storage::{JobStorage, SqliteStorage},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create storage
    let storage = SqliteStorage::new("sqlite:example.db").await?;

    // Create an SSH job payload
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
            secret: Some("enable_secret".to_string()),
        },
        commands: vec![
            "show version".to_string(),
            "show ip interface brief".to_string(),
            "show running-config | include hostname".to_string(),
        ],
        timeout: Some(Duration::from_secs(300)),
        retry_count: Some(0),
        description: Some("Basic device information gathering".to_string()),
    };

    println!("Created SSH job: {:?}", ssh_job);

    // In a real application, you would submit this job to Apalis
    // For this example, we'll just validate it
    scheduler::jobs::validate_ssh_job(&ssh_job)?;

    println!("Job validation passed!");

    // Test storage operations
    println!("Testing storage operations...");

    // List jobs (should be empty initially)
    let jobs = storage
        .list_jobs(scheduler::jobs::types::JobFilter::default())
        .await?;
    println!("Found {} jobs in storage", jobs.len());

    println!("Basic usage example completed successfully!");

    Ok(())
}
