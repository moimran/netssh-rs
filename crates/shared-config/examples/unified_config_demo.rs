use shared_config::WorkspaceConfig;

/// Example demonstrating unified configuration management across all crates
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Unified Configuration Demo");
    println!("=============================");

    // Load the unified configuration
    let config = WorkspaceConfig::load()?;

    println!("✅ Configuration loaded successfully from workspace root");

    // Display global configuration
    println!("\n🌍 Global Configuration:");
    println!("   - Environment: {}", config.global().environment);
    println!("   - Log Level: {}", config.global().log_level);
    println!("   - Default Timeout: {}s", config.global().default_timeout_seconds);

    // Display scheduler configuration
    println!("\n📋 Scheduler Configuration:");
    println!("   - Enabled: {}", config.scheduler().enabled);
    println!("   - Server: {}:{}", config.scheduler().server.host, config.scheduler().server.port);
    println!("   - Database: {}", config.scheduler().database.url);
    println!("   - Worker Concurrency: {}", config.scheduler().worker.concurrency);
    println!("   - Connection Reuse: {}", config.scheduler().worker.connection_reuse);
    println!("   - Max Idle Time: {}s", config.scheduler().worker.max_idle_time_seconds);
    println!("   - Failure Strategy: {}", config.scheduler().worker.failure_strategy);

    // Display netssh-core configuration
    println!("\n🔌 NetSSH Core Configuration:");
    println!("   - Default SSH Timeout: {}s", config.netssh().default_ssh_timeout);
    println!("   - Default Command Timeout: {}s", config.netssh().default_command_timeout);
    println!("   - Default Port: {}", config.netssh().default_port);
    println!("   - Buffer Size: {} bytes", config.netssh().buffer_size);
    println!("   - Max Retries: {}", config.netssh().max_retries);
    println!("   - Connection Pool Size: {}", config.netssh().connection_pool_size);
    println!("   - Session Logging: {}", config.netssh().logging.session_logging);
    println!("   - Command Logging: {}", config.netssh().logging.command_logging);
    println!("   - Performance Logging: {}", config.netssh().logging.performance_logging);

    // Display TextFSM configuration
    println!("\n📝 TextFSM Configuration:");
    println!("   - Template Cache Size: {}", config.textfsm().template_cache_size);
    println!("   - Parsing Timeout: {}s", config.textfsm().parsing_timeout_seconds);
    println!("   - Template Directories: {:?}", config.textfsm().template_directories);
    println!("   - Enable Caching: {}", config.textfsm().enable_caching);

    println!("\n🎯 Benefits of Unified Configuration:");
    println!("   ✅ Single source of truth for all settings");
    println!("   ✅ Consistent configuration across all crates");
    println!("   ✅ Environment variable support with NETSSH_ prefix");
    println!("   ✅ Hierarchical structure for easy organization");
    println!("   ✅ Type-safe configuration with validation");
    println!("   ✅ Default values for all settings");

    println!("\n🔧 Environment Variable Examples:");
    println!("   export NETSSH_GLOBAL_LOG_LEVEL=debug");
    println!("   export NETSSH_SCHEDULER_WORKER_CONCURRENCY=8");
    println!("   export NETSSH_NETSSH_DEFAULT_SSH_TIMEOUT=60");
    println!("   export NETSSH_TEXTFSM_TEMPLATE_CACHE_SIZE=2000");

    println!("\n📁 Configuration File Location:");
    println!("   - Primary: ./config.toml (workspace root)");
    println!("   - Override: Environment variables with NETSSH_ prefix");

    Ok(())
}
