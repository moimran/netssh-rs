use std::time::Duration;
use uuid::Uuid;

use scheduler::{
    config::LoggingConfig,
    jobs::types::{SshConnectionConfig, SshJobPayload},
    logging,
    storage::SqliteStorage,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Demo different logging configurations
    println!("=== Logging Demo ===\n");

    // 1. Console logging with text format
    println!("1. Console logging (text format):");
    demo_console_text_logging().await?;

    println!("\n{}\n", "=".repeat(50));

    // 2. Console logging with JSON format
    println!("2. Console logging (JSON format):");
    demo_console_json_logging().await?;

    println!("\n{}\n", "=".repeat(50));

    // 3. File logging with rotation
    println!("3. File logging with rotation:");
    demo_file_logging().await?;

    println!("\nLogging demo completed! Check the generated log files.");
    Ok(())
}

async fn demo_console_text_logging() -> Result<(), Box<dyn std::error::Error>> {
    let config = LoggingConfig {
        level: "info".to_string(),
        file: None,
        format: Some("text".to_string()),
        rotation: None,
    };

    logging::init_logging(&config)?;

    // Create some sample operations to generate logs
    let _storage = SqliteStorage::new("sqlite:demo_console_text.db").await?;

    // Create a sample SSH job
    let ssh_job = create_sample_ssh_job();

    // Validate the job (this will generate validation logs)
    scheduler::jobs::validate_ssh_job(&ssh_job)?;

    println!("✓ Text console logging demo completed");
    Ok(())
}

async fn demo_console_json_logging() -> Result<(), Box<dyn std::error::Error>> {
    let config = LoggingConfig {
        level: "debug".to_string(),
        file: None,
        format: Some("json".to_string()),
        rotation: None,
    };

    logging::init_logging(&config)?;

    // Create some sample operations to generate logs
    let _storage = SqliteStorage::new("sqlite:demo_console_json.db").await?;

    // Create a sample SSH job
    let ssh_job = create_sample_ssh_job();

    // Validate the job (this will generate validation logs)
    scheduler::jobs::validate_ssh_job(&ssh_job)?;

    println!("✓ JSON console logging demo completed");
    Ok(())
}

async fn demo_file_logging() -> Result<(), Box<dyn std::error::Error>> {
    let config = LoggingConfig {
        level: "trace".to_string(),
        file: Some("/tmp/scheduler_demo.log".to_string()),
        format: Some("json".to_string()),
        rotation: Some("daily".to_string()),
    };

    // Create logs directory if it doesn't exist
    std::fs::create_dir_all("/tmp")?;

    logging::init_logging(&config)?;

    // Create some sample operations to generate logs
    let _storage = SqliteStorage::new("sqlite:demo_file.db").await?;

    // Create multiple sample SSH jobs to generate more logs
    for i in 1..=3 {
        let mut ssh_job = create_sample_ssh_job();
        ssh_job.connection.host = format!("192.168.1.{}", i);
        ssh_job.commands = vec![
            format!("show version # Device {}", i),
            format!("show ip interface brief # Device {}", i),
        ];

        // Validate the job (this will generate validation logs)
        scheduler::jobs::validate_ssh_job(&ssh_job)?;

        // Simulate some delay
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    println!("✓ File logging demo completed - check /tmp/scheduler_demo.log");
    Ok(())
}

fn create_sample_ssh_job() -> SshJobPayload {
    SshJobPayload {
        id: Uuid::new_v4(),
        connection: SshConnectionConfig {
            host: "192.168.1.100".to_string(),
            username: "admin".to_string(),
            password: Some("password".to_string()),
            private_key: None,
            port: Some(22),
            device_type: "cisco_ios".to_string(),
            timeout_seconds: Some(60),
            secret: Some("demo_enable_secret".to_string()),
        },
        commands: vec![
            "show version".to_string(),
            "show ip interface brief".to_string(),
            "show running-config | include hostname".to_string(),
        ],
        timeout: Some(Duration::from_secs(300)),
        retry_count: Some(0),
        description: Some("Demo SSH job for logging".to_string()),
    }
}
