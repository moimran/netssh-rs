use netssh_core::settings::{
    get_buffer_setting, get_concurrency_setting, get_network_timeout, get_ssh_timeout,
    BufferSettingType, ConcurrencySettingType, NetworkTimeoutType, Settings, SshTimeoutType,
};
use std::fs;
use std::time::Duration;

#[test]
fn test_settings_default() {
    let settings = Settings::default();

    // Test default network settings
    assert_eq!(settings.network.tcp_connect_timeout_secs, 60);
    assert_eq!(settings.network.default_ssh_port, 22);
    assert_eq!(settings.network.max_retry_attempts, 3);

    // Test default SSH settings
    assert_eq!(settings.ssh.blocking_timeout_secs, 30);
    assert_eq!(settings.ssh.keepalive_interval_secs, 60);

    // Test default buffer settings
    assert_eq!(settings.buffer.read_buffer_size, 65536);
    assert_eq!(settings.buffer.buffer_pool_size, 32);
    assert!(settings.buffer.auto_clear_buffer);

    // Test default concurrency settings
    assert_eq!(settings.concurrency.max_connections, 100);
    assert_eq!(settings.concurrency.permit_acquire_timeout_ms, 5000);

    // Test default logging settings
    assert_eq!(settings.logging.enable_session_log, false);
    assert_eq!(settings.logging.session_log_path, "logs");
}

#[test]
fn test_settings_from_json() {
    let json = r#"
    {
        "network": {
            "tcp_connect_timeout_secs": 30,
            "tcp_read_timeout_secs": 20,
            "tcp_write_timeout_secs": 25,
            "command_response_timeout_secs": 40,
            "pattern_match_timeout_secs": 15,
            "command_exec_delay_ms": 100,
            "retry_delay_ms": 1000,
            "device_operation_timeout_secs": 60,
            "default_ssh_port": 2222,
            "max_retry_attempts": 5
        },
        "ssh": {
            "blocking_timeout_secs": 15,
            "keepalive_interval_secs": 30,
            "auth_timeout_secs": 20,
            "channel_open_timeout_secs": 25
        },
        "buffer": {
            "read_buffer_size": 32768,
            "buffer_pool_size": 16,
            "auto_clear_buffer": false,
            "buffer_reuse_threshold": 8192
        },
        "concurrency": {
            "max_connections": 50,
            "permit_acquire_timeout_ms": 2000,
            "connection_idle_timeout_secs": 300
        },
        "logging": {
            "enable_session_log": true,
            "session_log_path": "custom_logs",
            "log_binary_data": false
        }
    }
    "#;

    let settings = Settings::load_from_json(json).unwrap();

    // Test custom network settings
    assert_eq!(settings.network.tcp_connect_timeout_secs, 30);
    assert_eq!(settings.network.tcp_read_timeout_secs, 20);
    assert_eq!(settings.network.tcp_write_timeout_secs, 25);
    assert_eq!(settings.network.command_response_timeout_secs, 40);
    assert_eq!(settings.network.pattern_match_timeout_secs, 15);
    assert_eq!(settings.network.command_exec_delay_ms, 100);
    assert_eq!(settings.network.retry_delay_ms, 1000);
    assert_eq!(settings.network.device_operation_timeout_secs, 60);
    assert_eq!(settings.network.default_ssh_port, 2222);
    assert_eq!(settings.network.max_retry_attempts, 5);

    // Test custom SSH settings
    assert_eq!(settings.ssh.blocking_timeout_secs, 15);
    assert_eq!(settings.ssh.keepalive_interval_secs, 30);
    assert_eq!(settings.ssh.auth_timeout_secs, 20);
    assert_eq!(settings.ssh.channel_open_timeout_secs, 25);

    // Test custom buffer settings
    assert_eq!(settings.buffer.read_buffer_size, 32768);
    assert_eq!(settings.buffer.buffer_pool_size, 16);
    assert_eq!(settings.buffer.auto_clear_buffer, false);
    assert_eq!(settings.buffer.buffer_reuse_threshold, 8192);

    // Test custom concurrency settings
    assert_eq!(settings.concurrency.max_connections, 50);
    assert_eq!(settings.concurrency.permit_acquire_timeout_ms, 2000);
    assert_eq!(settings.concurrency.connection_idle_timeout_secs, 300);

    // Test custom logging settings
    assert_eq!(settings.logging.enable_session_log, true);
    assert_eq!(settings.logging.session_log_path, "custom_logs");
    assert_eq!(settings.logging.log_binary_data, false);
}

#[test]
fn test_settings_from_file() {
    // Create a temporary settings file
    let temp_file = "temp_settings.json";
    let json = r#"
    {
        "network": {
            "tcp_connect_timeout_secs": 45,
            "tcp_read_timeout_secs": 25,
            "tcp_write_timeout_secs": 30,
            "command_response_timeout_secs": 35,
            "pattern_match_timeout_secs": 20,
            "command_exec_delay_ms": 150,
            "retry_delay_ms": 2000,
            "device_operation_timeout_secs": 90,
            "default_ssh_port": 8022,
            "max_retry_attempts": 3
        },
        "ssh": {
            "blocking_timeout_secs": 25,
            "auth_timeout_secs": 30,
            "keepalive_interval_secs": 45,
            "channel_open_timeout_secs": 35
        },
        "buffer": {
            "read_buffer_size": 16384,
            "buffer_pool_size": 24,
            "buffer_reuse_threshold": 4096,
            "auto_clear_buffer": true
        },
        "concurrency": {
            "max_connections": 75,
            "permit_acquire_timeout_ms": 3000,
            "connection_idle_timeout_secs": 600
        },
        "logging": {
            "enable_session_log": true,
            "session_log_path": "custom_logs",
            "log_binary_data": true
        }
    }
    "#;

    fs::write(temp_file, json).unwrap();

    // Load settings from the file
    let settings = Settings::load_from_file(temp_file).unwrap();

    // Cleanup
    fs::remove_file(temp_file).unwrap();

    // Test settings loaded from file
    assert_eq!(settings.network.tcp_connect_timeout_secs, 45);
    assert_eq!(settings.network.tcp_read_timeout_secs, 25);
    assert_eq!(settings.network.tcp_write_timeout_secs, 30);
    assert_eq!(settings.network.command_response_timeout_secs, 35);
    assert_eq!(settings.network.pattern_match_timeout_secs, 20);
    assert_eq!(settings.network.command_exec_delay_ms, 150);
    assert_eq!(settings.network.retry_delay_ms, 2000);
    assert_eq!(settings.network.device_operation_timeout_secs, 90);
    assert_eq!(settings.network.default_ssh_port, 8022);
    assert_eq!(settings.network.max_retry_attempts, 3);

    // Test custom SSH settings
    assert_eq!(settings.ssh.blocking_timeout_secs, 25);
    assert_eq!(settings.ssh.auth_timeout_secs, 30);
    assert_eq!(settings.ssh.keepalive_interval_secs, 45);
    assert_eq!(settings.ssh.channel_open_timeout_secs, 35);

    // Test custom buffer settings
    assert_eq!(settings.buffer.read_buffer_size, 16384);
    assert_eq!(settings.buffer.buffer_pool_size, 24);
    assert_eq!(settings.buffer.buffer_reuse_threshold, 4096);
    assert_eq!(settings.buffer.auto_clear_buffer, true);

    assert_eq!(settings.concurrency.max_connections, 75);
    assert_eq!(settings.concurrency.permit_acquire_timeout_ms, 3000);
    assert_eq!(settings.concurrency.connection_idle_timeout_secs, 600);
    assert_eq!(settings.logging.enable_session_log, true);
    assert_eq!(settings.logging.session_log_path, "custom_logs");
    assert_eq!(settings.logging.log_binary_data, true);
}

#[test]
fn test_settings_file_not_found() {
    let result = Settings::load_from_file("nonexistent_file.json");
    assert!(result.is_err());
}

#[test]
fn test_settings_invalid_json() {
    let invalid_json = r#"
    {
        "network": {
            "tcp_connect_timeout_secs": "not a number"
        }
    }
    "#;

    let result = Settings::load_from_json(invalid_json);
    assert!(result.is_err());
}

#[test]
fn test_get_network_timeout() {
    // Initialize settings first
    let _ = Settings::init(None);

    // Test getting different network timeout types
    let tcp_connect = get_network_timeout(NetworkTimeoutType::TcpConnect);
    assert_eq!(tcp_connect, Duration::from_secs(60));

    let tcp_read = get_network_timeout(NetworkTimeoutType::TcpRead);
    assert_eq!(tcp_read, Duration::from_secs(30));

    let pattern_match = get_network_timeout(NetworkTimeoutType::PatternMatch);
    assert_eq!(pattern_match, Duration::from_secs(20));
}

#[test]
fn test_get_ssh_timeout() {
    // Initialize settings first
    let _ = Settings::init(None);

    // Test getting different SSH timeout types
    let blocking = get_ssh_timeout(SshTimeoutType::Blocking);
    assert_eq!(blocking, Duration::from_secs(30));

    let keepalive = get_ssh_timeout(SshTimeoutType::KeepaliveInterval);
    assert_eq!(keepalive, Duration::from_secs(60));
}

#[test]
fn test_get_concurrency_setting() {
    // Initialize settings first
    let _ = Settings::init(None);

    // Test getting different concurrency settings
    let max_connections = get_concurrency_setting(ConcurrencySettingType::MaxConnections);
    assert_eq!(max_connections, 100);

    let permit_timeout = get_concurrency_setting(ConcurrencySettingType::PermitAcquireTimeoutMs);
    assert_eq!(permit_timeout, 5000);
}

#[test]
fn test_get_buffer_setting() {
    // Initialize settings first
    let _ = Settings::init(None);

    // Test getting different buffer settings
    let buffer_size = get_buffer_setting(BufferSettingType::ReadBufferSize);
    assert_eq!(buffer_size, 65536);

    let pool_size = get_buffer_setting(BufferSettingType::BufferPoolSize);
    assert_eq!(pool_size, 32);
}

#[test]
fn test_settings_update() {
    // Initialize settings first
    let _ = Settings::init(None);

    // Update a specific setting
    Settings::update(|s| {
        s.network.tcp_connect_timeout_secs = 120;
        s.buffer.read_buffer_size = 131072;
    })
    .unwrap();

    // Verify the update worked
    let timeout = get_network_timeout(NetworkTimeoutType::TcpConnect);
    assert_eq!(timeout, Duration::from_secs(120));

    let buffer_size = get_buffer_setting(BufferSettingType::ReadBufferSize);
    assert_eq!(buffer_size, 131072);
}

#[test]
fn test_settings_init_from_workspace_config() {
    // Test initialization from workspace config
    // This should work even if config.toml doesn't exist (uses defaults)
    let result = Settings::init_from_workspace_config();

    // Should succeed (may use defaults if config file not found)
    assert!(result.is_ok(), "Failed to initialize from workspace config: {:?}", result);

    // Verify we can get settings values
    let timeout = get_network_timeout(NetworkTimeoutType::TcpConnect);
    assert!(timeout.as_secs() > 0);

    let buffer_size = get_buffer_setting(BufferSettingType::ReadBufferSize);
    assert!(buffer_size > 0);
}
