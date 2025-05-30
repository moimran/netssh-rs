use shared_config::WorkspaceConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Cross-Crate Integration Demo");
    println!("================================");

    // Load the unified configuration
    let config = WorkspaceConfig::load().unwrap_or_else(|e| {
        println!("⚠️  Failed to load config: {}. Using defaults.", e);
        create_default_config()
    });

    println!("✅ Configuration loaded successfully\n");

    // Demonstrate how each crate would access its configuration
    demonstrate_netssh_core_integration(&config)?;
    demonstrate_textfsm_integration(&config)?;
    demonstrate_scheduler_integration(&config)?;

    println!("\n🎉 Configuration Integration Complete!");
    println!("   ✅ netssh-core: Can access SSH connection settings");
    println!("   ✅ netssh-textfsm: Can access TextFSM parsing settings");
    println!("   ✅ scheduler: Can access job scheduling settings");
    println!("   ✅ All crates share the same global configuration");

    Ok(())
}

fn demonstrate_netssh_core_integration(config: &WorkspaceConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔌 NetSSH Core Integration");
    println!("---------------------------");
    
    // Access netssh configuration
    let netssh_config = config.netssh();
    
    println!("   📡 SSH Connection Settings:");
    println!("      - Default SSH Timeout: {}s", netssh_config.default_ssh_timeout);
    println!("      - Default Command Timeout: {}s", netssh_config.default_command_timeout);
    println!("      - Default Port: {}", netssh_config.default_port);
    println!("      - Buffer Size: {} bytes", netssh_config.buffer_size);
    println!("      - Max Retries: {}", netssh_config.max_retries);
    println!("      - Connection Pool Size: {}", netssh_config.connection_pool_size);
    
    println!("   📊 Logging Configuration:");
    println!("      - Session Logging: {}", netssh_config.logging.session_logging);
    println!("      - Command Logging: {}", netssh_config.logging.command_logging);
    println!("      - Performance Logging: {}", netssh_config.logging.performance_logging);
    println!("      - Debug Mode: {}", netssh_config.logging.debug_mode);
    
    println!("   🔒 Security Configuration:");
    println!("      - Strict Host Key Checking: {}", netssh_config.security.strict_host_key_checking);
    println!("      - Known Hosts File: {}", netssh_config.security.known_hosts_file);
    println!("      - Max Auth Attempts: {}", netssh_config.security.max_auth_attempts);

    // Show how netssh-core would use these settings
    println!("   🔧 How netssh-core uses these settings:");
    println!("      ✅ SSH connections would use timeout: {}s", netssh_config.default_ssh_timeout);
    println!("      ✅ Commands would use timeout: {}s", netssh_config.default_command_timeout);
    println!("      ✅ Default port: {}", netssh_config.default_port);
    println!("      ✅ Buffer size: {} bytes", netssh_config.buffer_size);
    println!("      ✅ Connection pool size: {}", netssh_config.connection_pool_size);
    
    Ok(())
}

fn demonstrate_textfsm_integration(config: &WorkspaceConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📝 TextFSM Integration");
    println!("----------------------");

    // Access TextFSM configuration from shared config
    let textfsm_config = config.textfsm();

    println!("   🎯 TextFSM Configuration:");
    println!("      - Template Cache Size: {}", textfsm_config.template_cache_size);
    println!("      - Parsing Timeout: {}s", textfsm_config.parsing_timeout_seconds);
    println!("      - Enable Caching: {}", textfsm_config.enable_caching);
    println!("      - Template Directories:");
    for (i, dir) in textfsm_config.template_directories.iter().enumerate() {
        println!("        {}. {}", i + 1, dir);
    }

    // Show how netssh-textfsm would use these settings
    println!("   🔧 How netssh-textfsm uses these settings:");
    println!("      ✅ Template cache would hold {} templates", textfsm_config.template_cache_size);
    println!("      ✅ Parsing would timeout after {}s", textfsm_config.parsing_timeout_seconds);
    println!("      ✅ Caching is {}", if textfsm_config.enable_caching { "enabled" } else { "disabled" });
    println!("      ✅ Templates searched in {} directories", textfsm_config.template_directories.len());

    Ok(())
}

fn demonstrate_scheduler_integration(config: &WorkspaceConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Scheduler Integration");
    println!("------------------------");

    // Access scheduler configuration from shared config
    let scheduler_config = config.scheduler();

    println!("   🗄️  Database Configuration:");
    println!("      - URL: {}", scheduler_config.database.url);
    println!("      - Max Connections: {}", scheduler_config.database.max_connections);

    println!("   🌐 Server Configuration:");
    println!("      - Host: {}", scheduler_config.server.host);
    println!("      - Port: {}", scheduler_config.server.port);
    println!("      - Bind Address: {}:{}", scheduler_config.server.host, scheduler_config.server.port);

    println!("   👷 Worker Configuration:");
    println!("      - Concurrency: {}", scheduler_config.worker.concurrency);
    println!("      - Timeout: {}s", scheduler_config.worker.timeout_seconds);
    println!("      - Connection Reuse: {}", scheduler_config.worker.connection_reuse);
    println!("      - Max Idle Time: {}s", scheduler_config.worker.max_idle_time_seconds);
    println!("      - Max Connections per Worker: {}", scheduler_config.worker.max_connections_per_worker);
    println!("      - Failure Strategy: {}", scheduler_config.worker.failure_strategy);

    println!("   📊 Board Configuration:");
    println!("      - Enabled: {}", scheduler_config.board.enabled);
    println!("      - UI Path: {}", scheduler_config.board.ui_path);
    println!("      - API Prefix: {}", scheduler_config.board.api_prefix);
    println!("      - Auth Enabled: {}", scheduler_config.board.auth_enabled);

    println!("   📝 Logging Configuration:");
    println!("      - Level: {}", scheduler_config.logging.level);
    println!("      - File: {:?}", scheduler_config.logging.file);

    println!("   ⏰ Scheduler Service Configuration:");
    println!("      - Enabled: {}", scheduler_config.scheduler.enabled);
    println!("      - Poll Interval: {}s", scheduler_config.scheduler.poll_interval_seconds);
    println!("      - Timezone: {:?}", scheduler_config.scheduler.timezone);
    println!("      - Max Concurrent Jobs: {}", scheduler_config.scheduler.max_concurrent_jobs);

    // Show how scheduler would use these settings
    println!("   🔧 How scheduler uses these settings:");
    println!("      ✅ Database connections limited to {}", scheduler_config.database.max_connections);
    println!("      ✅ Server would bind to {}:{}", scheduler_config.server.host, scheduler_config.server.port);
    println!("      ✅ Workers would run with concurrency {}", scheduler_config.worker.concurrency);
    println!("      ✅ Jobs would timeout after {}s", scheduler_config.worker.timeout_seconds);
    println!("      ✅ Connection reuse is {}", if scheduler_config.worker.connection_reuse { "enabled" } else { "disabled" });

    Ok(())
}

fn create_default_config() -> WorkspaceConfig {
    // This would normally use WorkspaceConfig::default() if it existed
    // For now, we'll create a minimal config
    shared_config::WorkspaceConfig {
        global: shared_config::GlobalConfig {
            log_level: "info".to_string(),
            environment: "development".to_string(),
            default_timeout_seconds: 30,
        },
        scheduler: shared_config::SchedulerConfig {
            enabled: true,
            database: shared_config::DatabaseConfig {
                url: "sqlite:scheduler.db".to_string(),
                max_connections: 10,
            },
            server: shared_config::ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
            },
            worker: shared_config::WorkerConfig {
                concurrency: 4,
                timeout_seconds: 300,
                connection_reuse: true,
                max_idle_time_seconds: 300,
                max_connections_per_worker: 10,
                failure_strategy: shared_config::FailureStrategy::ContinueOnFailure,
                failure_strategy_n: 3,
            },
            board: shared_config::BoardConfig {
                enabled: true,
                ui_path: "/board".to_string(),
                api_prefix: "/board/api".to_string(),
                auth_enabled: false,
            },
            logging: shared_config::LoggingConfig {
                level: "info".to_string(),
                file: None,
                format: None,
                rotation: None,
            },
            scheduler: shared_config::SchedulerServiceConfig {
                enabled: true,
                poll_interval_seconds: 30,
                timezone: Some("UTC".to_string()),
                max_concurrent_jobs: 10,
            },
        },
        netssh: shared_config::NetsshConfig {
            default_ssh_timeout: 30,
            default_command_timeout: 60,
            default_port: 22,
            buffer_size: 16384,
            max_retries: 3,
            connection_pool_size: 50,
            logging: shared_config::NetsshLoggingConfig {
                session_logging: false,
                command_logging: true,
                performance_logging: true,
                debug_mode: false,
            },
            security: shared_config::SecurityConfig {
                strict_host_key_checking: true,
                known_hosts_file: "~/.ssh/known_hosts".to_string(),
                max_auth_attempts: 3,
            },
        },
        textfsm: shared_config::TextFsmConfig {
            template_cache_size: 1000,
            parsing_timeout_seconds: 10,
            template_directories: vec!["templates/".to_string(), "/usr/share/textfsm/".to_string()],
            enable_caching: true,
        },
    }
}
