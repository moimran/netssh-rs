use netssh_core::settings::{
    Settings, NetworkTimeoutType, SshTimeoutType, 
    ConcurrencySettingType, BufferSettingType,
    get_network_timeout, get_ssh_timeout, 
    get_concurrency_setting, get_buffer_setting
};
use std::time::Duration;
use std::fs;
use std::path::Path;

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
            "default_ssh_port": 2222,
            "max_retry_attempts": 5
        },
        "ssh": {
            "blocking_timeout_secs": 15,
            "keepalive_interval_secs": 30
        },
        "buffer": {
            "read_buffer_size": 32768,
            "buffer_pool_size": 16,
            "auto_clear_buffer": false
        },
        "concurrency": {
            "max_connections": 50,
            "permit_acquire_timeout_ms": 2000
        },
        "logging": {
            "enable_session_log": true,
            "session_log_path": "custom_logs"
        }
    }
    "#;
    
    let settings = Settings::load_from_json(json).unwrap();
    
    // Test custom network settings
    assert_eq!(settings.network.tcp_connect_timeout_secs, 30);
    assert_eq!(settings.network.default_ssh_port, 2222);
    assert_eq!(settings.network.max_retry_attempts, 5);
    
    // Test custom SSH settings
    assert_eq!(settings.ssh.blocking_timeout_secs, 15);
    assert_eq!(settings.ssh.keepalive_interval_secs, 30);
    
    // Test custom buffer settings
    assert_eq!(settings.buffer.read_buffer_size, 32768);
    assert_eq!(settings.buffer.buffer_pool_size, 16);
    assert_eq!(settings.buffer.auto_clear_buffer, false);
    
    // Test custom concurrency settings
    assert_eq!(settings.concurrency.max_connections, 50);
    assert_eq!(settings.concurrency.permit_acquire_timeout_ms, 2000);
    
    // Test custom logging settings
    assert_eq!(settings.logging.enable_session_log, true);
    assert_eq!(settings.logging.session_log_path, "custom_logs");
}

#[test]
fn test_settings_from_file() {
    // Create a temporary settings file
    let temp_file = "temp_settings.json";
    let json = r#"
    {
        "network": {
            "tcp_connect_timeout_secs": 45,
            "default_ssh_port": 8022
        },
        "ssh": {
            "blocking_timeout_secs": 25
        },
        "buffer": {
            "read_buffer_size": 16384
        },
        "concurrency": {
            "max_connections": 75
        },
        "logging": {
            "enable_session_log": true
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
    assert_eq!(settings.network.default_ssh_port, 8022);
    assert_eq!(settings.ssh.blocking_timeout_secs, 25);
    assert_eq!(settings.buffer.read_buffer_size, 16384);
    assert_eq!(settings.concurrency.max_connections, 75);
    assert_eq!(settings.logging.enable_session_log, true);
    
    // Other settings should still have defaults
    assert_eq!(settings.network.max_retry_attempts, 3);
    assert_eq!(settings.ssh.keepalive_interval_secs, 60);
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
    }).unwrap();
    
    // Verify the update worked
    let timeout = get_network_timeout(NetworkTimeoutType::TcpConnect);
    assert_eq!(timeout, Duration::from_secs(120));
    
    let buffer_size = get_buffer_setting(BufferSettingType::ReadBufferSize);
    assert_eq!(buffer_size, 131072);
} 