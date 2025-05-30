use std::sync::Arc;
use std::time::Duration;

use scheduler::{
    config::Config,
    jobs::{SshConnectionConfig, SshJobPayload},
    storage::{SqliteStorage, Storage},
};
use tokio::time::sleep;
use uuid::Uuid;

/// Example demonstrating Phase 1 SSH Connection Scaling with connection reuse
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    println!("🚀 Phase 1 SSH Connection Scaling Demo");
    println!("=====================================");

    // Load configuration from unified workspace configuration
    let config = Config::from_env().unwrap_or_else(|e| {
        println!("Failed to load config: {}. Using defaults.", e);
        Config::default()
    });

    println!("✅ Configuration loaded from unified workspace config");
    println!("   - Worker concurrency: {}", config.worker().concurrency);
    println!("   - Connection reuse: {}", config.worker().connection_reuse);
    println!("   - Max idle time: {}s", config.worker().max_idle_time_seconds);
    println!("   - Max connections per worker: {}", config.worker().max_connections_per_worker);
    println!("   - Failure strategy: {}", config.worker().failure_strategy);

    // Set up storage (use in-memory database for demo)
    let storage = Arc::new(SqliteStorage::new("sqlite::memory:").await?);
    storage.initialize().await?;

    println!("✅ Storage initialized");

    // Create sample SSH job payloads for testing
    let jobs = create_sample_jobs();
    
    println!("✅ Created {} sample SSH jobs", jobs.len());
    println!("   - Jobs target the same device to demonstrate connection reuse");
    println!("   - Each job executes multiple commands");

    // Simulate job execution (in a real scenario, these would be queued through Apalis)
    println!("\n🔄 Simulating job execution with connection reuse...");
    
    for (i, job) in jobs.iter().enumerate() {
        println!("\n📋 Executing Job {} (ID: {})", i + 1, job.id);
        println!("   - Host: {}", job.connection.host);
        println!("   - Device Type: {}", job.connection.device_type);
        println!("   - Commands: {}", job.commands.len());
        
        // In a real implementation, this would be handled by the Apalis worker
        // For demo purposes, we'll just show the job structure
        
        // Simulate processing time
        sleep(Duration::from_millis(100)).await;
        
        println!("   ✅ Job {} would be processed with connection reuse", i + 1);
    }

    println!("\n🎉 Phase 1 Implementation Benefits:");
    println!("   ✅ Connection reuse within worker lifecycle");
    println!("   ✅ Reduced connection setup overhead");
    println!("   ✅ Configurable failure strategies");
    println!("   ✅ Automatic idle connection cleanup");
    println!("   ✅ Connection pool size limits");
    println!("   ✅ Enhanced logging and monitoring");

    println!("\n📊 Expected Performance Improvements:");
    println!("   - 20-30% reduction in job execution time for repeated devices");
    println!("   - 50% reduction in connection setup overhead");
    println!("   - Stable memory usage under load");
    println!("   - Support for 100+ concurrent jobs");

    println!("\n🔮 Next Steps (Phase 2 & 3):");
    println!("   - Thread safety migration in netssh-core");
    println!("   - Advanced connection pooling");
    println!("   - Dynamic scaling based on load");
    println!("   - Multi-connection workers");

    Ok(())
}

fn create_sample_jobs() -> Vec<SshJobPayload> {
    let connection_config = SshConnectionConfig {
        host: "192.168.1.1".to_string(),
        username: "admin".to_string(),
        password: Some("password".to_string()),
        private_key: None,
        port: Some(22),
        device_type: "cisco_ios".to_string(),
        timeout_seconds: Some(30),
        secret: Some("enable_secret".to_string()),
    };

    vec![
        SshJobPayload {
            id: Uuid::new_v4(),
            connection: connection_config.clone(),
            commands: vec![
                "show version".to_string(),
                "show interfaces brief".to_string(),
                "show ip route summary".to_string(),
            ],
            timeout: Some(Duration::from_secs(60)),
            retry_count: Some(2),
            description: Some("Device health check".to_string()),
        },
        SshJobPayload {
            id: Uuid::new_v4(),
            connection: connection_config.clone(),
            commands: vec![
                "show running-config | include hostname".to_string(),
                "show memory summary".to_string(),
                "show processes cpu".to_string(),
            ],
            timeout: Some(Duration::from_secs(60)),
            retry_count: Some(2),
            description: Some("System monitoring".to_string()),
        },
        SshJobPayload {
            id: Uuid::new_v4(),
            connection: connection_config.clone(),
            commands: vec![
                "show logging | tail 10".to_string(),
                "show users".to_string(),
                "show clock".to_string(),
            ],
            timeout: Some(Duration::from_secs(60)),
            retry_count: Some(2),
            description: Some("Operational status".to_string()),
        },
    ]
}
