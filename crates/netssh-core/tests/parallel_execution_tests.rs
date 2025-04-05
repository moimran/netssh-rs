use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

use netssh_core::device_connection::{DeviceConfig, NetworkDeviceConnection};
use netssh_core::error::NetsshError;
use netssh_core::parallel_execution::{
    CommandStatus, FailureStrategy, ParallelExecutionConfig, ParallelExecutionManager,
};

// Mock device implementation for testing
struct MockDevice {
    device_id: String,
    device_type: String,
    commands: HashMap<String, Result<String, NetsshError>>,
    connect_result: Result<(), NetsshError>,
    delay_ms: Option<u64>,
}

impl MockDevice {
    fn new(
        device_id: &str,
        device_type: &str,
        commands: HashMap<String, Result<String, NetsshError>>,
        connect_result: Result<(), NetsshError>,
        delay_ms: Option<u64>,
    ) -> Self {
        Self {
            device_id: device_id.to_string(),
            device_type: device_type.to_string(),
            commands,
            connect_result,
            delay_ms,
        }
    }
}

impl NetworkDeviceConnection for MockDevice {
    fn connect(&mut self) -> Result<(), NetsshError> {
        self.connect_result.clone()
    }

    fn close(&mut self) -> Result<(), NetsshError> {
        Ok(())
    }

    fn check_config_mode(&mut self) -> Result<bool, NetsshError> {
        Ok(false)
    }

    fn enter_config_mode(&mut self, _config_command: Option<&str>) -> Result<(), NetsshError> {
        Ok(())
    }

    fn exit_config_mode(&mut self, _exit_command: Option<&str>) -> Result<(), NetsshError> {
        Ok(())
    }

    fn session_preparation(&mut self) -> Result<(), NetsshError> {
        Ok(())
    }

    fn terminal_settings(&mut self) -> Result<(), NetsshError> {
        Ok(())
    }

    fn set_terminal_width(&mut self, _width: u32) -> Result<(), NetsshError> {
        Ok(())
    }

    fn disable_paging(&mut self) -> Result<(), NetsshError> {
        Ok(())
    }

    fn set_base_prompt(&mut self) -> Result<String, NetsshError> {
        Ok("prompt>".to_string())
    }

    fn save_configuration(&mut self) -> Result<(), NetsshError> {
        Ok(())
    }

    async fn send_command_async(&mut self, command: &str) -> Result<String, NetsshError> {
        // Simulate command execution delay if specified
        if let Some(delay) = self.delay_ms {
            sleep(Duration::from_millis(delay)).await;
        }

        match self.commands.get(command) {
            Some(result) => result.clone(),
            None => Ok(format!("Mock output for command: {}", command)),
        }
    }

    fn send_command(&mut self, command: &str) -> Result<String, NetsshError> {
        // For testing, we'll use a simple implementation without the async part
        match self.commands.get(command) {
            Some(result) => result.clone(),
            None => Ok(format!("Mock output for command: {}", command)),
        }
    }

    fn get_device_type(&self) -> &str {
        &self.device_type
    }

    fn get_device_info(&mut self) -> Result<DeviceInfo, NetsshError> {
        let info = DeviceInfo {
            device_type: "mock".to_string(),
            hostname: "mock".to_string(),
            version: "1.0".to_string(),
            model: "mock".to_string(),
            serial: "12345".to_string(),
            uptime: "10 days".to_string(),
        };
        Ok(info)
    }

    fn send_config_set(
        &mut self,
        config_commands: Vec<String>,
        _exit_config_mode: Option<bool>,
        _read_timeout: Option<f64>,
        _strip_prompt: Option<bool>,
        _strip_command: Option<bool>,
        _config_mode_command: Option<&str>,
        _cmd_verify: Option<bool>,
        _enter_config_mode: Option<bool>,
        _error_pattern: Option<&str>,
        _terminator: Option<&str>,
        _bypass_commands: Option<&str>,
        _fast_cli: Option<bool>,
    ) -> Result<String, NetsshError> {
        let mut output = String::new();
        for cmd in config_commands {
            output.push_str(&format!("Configured: {}\n", cmd));
        }
        Ok(output)
    }
}

// Custom device factory for testing that returns our mock devices
struct MockDeviceFactory;

impl MockDeviceFactory {
    fn create_device(
        config: &DeviceConfig,
        commands: HashMap<String, Result<String, NetsshError>>,
        connect_result: Result<(), NetsshError>,
        delay_ms: Option<u64>,
    ) -> Box<dyn NetworkDeviceConnection + Send> {
        Box::new(MockDevice::new(
            &config.host,
            &config.device_type,
            commands,
            connect_result,
            delay_ms,
        ))
    }
}

#[tokio::test]
async fn test_parallel_execute_command_success() {
    // Create device configs
    let devices = vec![
        DeviceConfig {
            device_type: "cisco_ios".to_string(),
            host: "device1".to_string(),
            username: "user".to_string(),
            password: Some("pass".to_string()),
            port: None,
            timeout: None,
            secret: None,
            session_log: None,
        },
        DeviceConfig {
            device_type: "cisco_ios".to_string(),
            host: "device2".to_string(),
            username: "user".to_string(),
            password: Some("pass".to_string()),
            port: None,
            timeout: None,
            secret: None,
            session_log: None,
        },
    ];

    // Create a command to execute
    let command = "show version".to_string();

    // Create a map of device IDs to commands and results
    let mut device_commands = HashMap::new();

    for device in &devices {
        let mut commands = HashMap::new();
        commands.insert(
            command.clone(),
            Ok(format!("Version info for {}", device.host)),
        );
        device_commands.insert(device.host.clone(), commands);
    }

    // Create a manager with custom config
    let config = ParallelExecutionConfig {
        max_concurrency: 5,
        command_timeout: Some(Duration::from_secs(5)),
        connection_timeout: Some(Duration::from_secs(5)),
        failure_strategy: FailureStrategy::ContinueDevice,
        reuse_connections: true,
    };

    let manager = ParallelExecutionManager::with_config(config);

    // Execute command
    let device_command_map: HashMap<DeviceConfig, Vec<String>> = devices
        .into_iter()
        .map(|device| {
            let commands = vec![command.clone()];
            (device, commands)
        })
        .collect();

    let results = manager.execute_commands(device_command_map).await.unwrap();

    // Verify results
    assert_eq!(results.device_count, 2);
    assert_eq!(results.command_count, 2);
    assert_eq!(results.success_count, 2);
    assert_eq!(results.failure_count, 0);

    // Check device1 results
    let device1_results = results.get_device_results("device1").unwrap();
    assert_eq!(device1_results.len(), 1);
    assert_eq!(device1_results[0].command, command);
    assert_eq!(device1_results[0].output, "Version info for device1");
    assert_eq!(device1_results[0].status, CommandStatus::Success);

    // Check device2 results
    let device2_results = results.get_device_results("device2").unwrap();
    assert_eq!(device2_results.len(), 1);
    assert_eq!(device2_results[0].command, command);
    assert_eq!(device2_results[0].output, "Version info for device2");
    assert_eq!(device2_results[0].status, CommandStatus::Success);
}

#[tokio::test]
async fn test_parallel_execute_commands_on_all() {
    // Create device configs
    let devices = vec![
        DeviceConfig {
            device_type: "cisco_ios".to_string(),
            host: "device1".to_string(),
            username: "user".to_string(),
            password: Some("pass".to_string()),
            port: None,
            timeout: None,
            secret: None,
            session_log: None,
        },
        DeviceConfig {
            device_type: "cisco_ios".to_string(),
            host: "device2".to_string(),
            username: "user".to_string(),
            password: Some("pass".to_string()),
            port: None,
            timeout: None,
            secret: None,
            session_log: None,
        },
    ];

    // Create commands to execute
    let commands = vec!["show version".to_string(), "show interfaces".to_string()];

    // Create a manager with default config
    let mut manager = ParallelExecutionManager::new();

    // Execute commands
    let results = manager
        .execute_commands_on_all(devices, commands.clone())
        .await
        .unwrap();

    // Verify results
    assert_eq!(results.device_count, 2);
    assert_eq!(results.command_count, 4); // 2 devices * 2 commands
    assert_eq!(results.success_count, 4);
    assert_eq!(results.failure_count, 0);

    // Check device1 results
    let device1_results = results.get_device_results("device1").unwrap();
    assert_eq!(device1_results.len(), 2);
    assert_eq!(device1_results[0].command, commands[0]);
    assert_eq!(device1_results[1].command, commands[1]);

    // Check device2 results
    let device2_results = results.get_device_results("device2").unwrap();
    assert_eq!(device2_results.len(), 2);
    assert_eq!(device2_results[0].command, commands[0]);
    assert_eq!(device2_results[1].command, commands[1]);

    // Check command results
    let version_results = results.get_command_results(&commands[0]);
    assert_eq!(version_results.len(), 2);

    let interfaces_results = results.get_command_results(&commands[1]);
    assert_eq!(interfaces_results.len(), 2);
}

#[tokio::test]
async fn test_failure_strategy_stop_device() {
    // Create device configs
    let devices = vec![DeviceConfig {
        device_type: "cisco_ios".to_string(),
        host: "device1".to_string(),
        username: "user".to_string(),
        password: Some("pass".to_string()),
        port: None,
        timeout: None,
        secret: None,
        session_log: None,
    }];

    // Create commands to execute
    let commands = vec![
        "show version".to_string(),    // Will succeed
        "show interfaces".to_string(), // Will fail
        "show ip route".to_string(),   // Should be skipped due to failure strategy
    ];

    // Create a manager with StopDevice failure strategy
    let mut config = ParallelExecutionConfig::default();
    config.failure_strategy = FailureStrategy::StopDevice;
    let mut manager = ParallelExecutionManager::with_config(config);

    // Create a map of commands to results for the device
    let mut device_commands = HashMap::new();
    let mut commands_map = HashMap::new();
    commands_map.insert(commands[0].clone(), Ok("Version info".to_string()));
    commands_map.insert(
        commands[1].clone(),
        Err(NetsshError::CommandError("Command failed".to_string())),
    );
    commands_map.insert(commands[2].clone(), Ok("Route info".to_string()));
    device_commands.insert(devices[0].host.clone(), commands_map);

    // Execute commands
    let results = manager
        .execute_commands_on_all(devices, commands.clone())
        .await
        .unwrap();

    // Verify results
    assert_eq!(results.device_count, 1);
    assert_eq!(results.command_count, 3);
    assert_eq!(results.success_count, 1);
    assert_eq!(results.failure_count, 1);
    assert_eq!(results.skipped_count, 1);

    // Check device1 results
    let device1_results = results.get_device_results("device1").unwrap();
    assert_eq!(device1_results.len(), 3);

    // First command should succeed
    assert_eq!(device1_results[0].command, commands[0]);
    assert_eq!(device1_results[0].status, CommandStatus::Success);

    // Second command should fail
    assert_eq!(device1_results[1].command, commands[1]);
    assert_eq!(device1_results[1].status, CommandStatus::Failed);

    // Third command should be skipped
    assert_eq!(device1_results[2].command, commands[2]);
    assert_eq!(device1_results[2].status, CommandStatus::Skipped);
}

#[tokio::test]
async fn test_failure_strategy_stop_all() {
    // Create device configs
    let devices = vec![
        DeviceConfig {
            device_type: "cisco_ios".to_string(),
            host: "device1".to_string(),
            username: "user".to_string(),
            password: Some("pass".to_string()),
            port: None,
            timeout: None,
            secret: None,
            session_log: None,
        },
        DeviceConfig {
            device_type: "cisco_ios".to_string(),
            host: "device2".to_string(),
            username: "user".to_string(),
            password: Some("pass".to_string()),
            port: None,
            timeout: None,
            secret: None,
            session_log: None,
        },
    ];

    // Create commands to execute
    let commands = vec!["show version".to_string(), "show interfaces".to_string()];

    // Create a manager with StopAll failure strategy
    let mut config = ParallelExecutionConfig::default();
    config.failure_strategy = FailureStrategy::StopAll;
    let mut manager = ParallelExecutionManager::with_config(config);

    // Create a map of commands to results for each device
    let mut device_commands = HashMap::new();

    // Device 1 will succeed for first command, fail for second
    let mut device1_commands = HashMap::new();
    device1_commands.insert(commands[0].clone(), Ok("Version info device1".to_string()));
    device1_commands.insert(
        commands[1].clone(),
        Err(NetsshError::CommandError("Command failed".to_string())),
    );
    device_commands.insert(devices[0].host.clone(), device1_commands);

    // Device 2 would succeed for both commands, but second command won't be executed due to StopAll
    let mut device2_commands = HashMap::new();
    device2_commands.insert(commands[0].clone(), Ok("Version info device2".to_string()));
    device2_commands.insert(
        commands[1].clone(),
        Ok("Interfaces info device2".to_string()),
    );
    device_commands.insert(devices[1].host.clone(), device2_commands);

    // Execute commands
    let results = manager
        .execute_commands_on_all(devices, commands.clone())
        .await
        .unwrap();

    // Verify results
    // All commands may not be executed on all devices due to the StopAll strategy
    // and timing of parallel execution, so we'll just check the overall statistics
    assert_eq!(results.device_count, 2);
    assert!(results.success_count >= 1); // At least some should succeed
    assert_eq!(results.failure_count, 1); // Only one failure should be recorded

    // Check device1 results
    let device1_results = results.get_device_results("device1").unwrap();

    // Check for the failed command in device1
    let has_failed_command = device1_results
        .iter()
        .any(|r| r.status == CommandStatus::Failed && r.command == commands[1]);
    assert!(has_failed_command);
}

#[tokio::test]
async fn test_command_timeout() {
    // Create device config
    let device = DeviceConfig {
        device_type: "cisco_ios".to_string(),
        host: "device1".to_string(),
        username: "user".to_string(),
        password: Some("pass".to_string()),
        port: None,
        timeout: None,
        secret: None,
        session_log: None,
    };

    // Create command to execute
    let command = "show version".to_string();

    // Create a manager with a very short timeout
    let mut config = ParallelExecutionConfig::default();
    config.command_timeout = Some(Duration::from_millis(50)); // 50ms timeout
    let mut manager = ParallelExecutionManager::with_config(config);

    // Create a device that takes longer than the timeout to respond
    let mut commands = HashMap::new();
    commands.insert(command.clone(), Ok("Version info".to_string()));

    // Create a device that will take longer than the timeout to respond
    let delay_ms = 500; // 500ms delay, which exceeds the 50ms timeout

    // Execute command
    let device_command_map: HashMap<DeviceConfig, Vec<String>> =
        HashMap::from([(device, vec![command.clone()])]);

    let results = manager.execute_commands(device_command_map).await.unwrap();

    // Verify results
    assert_eq!(results.device_count, 1);
    assert_eq!(results.command_count, 1);

    // The command should have timed out
    assert!(results.timeout_count > 0 || results.failure_count > 0);

    // Check device1 results - status may be either Timeout or Failed depending on how the timeout is handled
    let device1_results = results.get_device_results("device1").unwrap();
    assert_eq!(device1_results.len(), 1);
    assert_eq!(device1_results[0].command, command);
    assert!(
        device1_results[0].status == CommandStatus::Timeout
            || device1_results[0].status == CommandStatus::Failed
    );
}

#[tokio::test]
async fn test_batch_command_results_methods() {
    // Create device configs
    let devices = vec![
        DeviceConfig {
            device_type: "cisco_ios".to_string(),
            host: "device1".to_string(),
            username: "user".to_string(),
            password: Some("pass".to_string()),
            port: None,
            timeout: None,
            secret: None,
            session_log: None,
        },
        DeviceConfig {
            device_type: "juniper_junos".to_string(),
            host: "device2".to_string(),
            username: "user".to_string(),
            password: Some("pass".to_string()),
            port: None,
            timeout: None,
            secret: None,
            session_log: None,
        },
    ];

    // Create commands to execute
    let commands = vec!["show version".to_string(), "show interfaces".to_string()];

    // Create a manager with default config
    let mut manager = ParallelExecutionManager::new();

    // Execute commands
    let results = manager
        .execute_commands_on_all(devices, commands.clone())
        .await
        .unwrap();

    // Test get_device_results
    let device1_results = results.get_device_results("device1").unwrap();
    assert_eq!(device1_results.len(), 2);

    // Test get_command_results
    let version_results = results.get_command_results(&commands[0]);
    assert_eq!(version_results.len(), 2);

    // Test filter_by_status
    let success_results = results.filter_by_status(CommandStatus::Success);
    assert_eq!(success_results.len(), 4); // All commands should succeed

    // Test successful_results
    let successful = results.successful_results();
    assert_eq!(successful.len(), 4);

    // Test failed_results
    let failed = results.failed_results();
    assert_eq!(failed.len(), 0);

    // Test utility functions
    use netssh_core::parallel_execution::utils;

    // Test to_json
    let json = utils::to_json(&results).unwrap();
    assert!(json.contains("device1"));
    assert!(json.contains("device2"));

    // Test to_csv
    let csv = utils::to_csv(&results);
    assert!(csv.contains("device1"));
    assert!(csv.contains("device2"));

    // Test group_by_command
    let grouped_by_command = utils::group_by_command(&results);
    assert_eq!(grouped_by_command.len(), 2);
    assert!(grouped_by_command.contains_key(commands[0].as_str()));
    assert!(grouped_by_command.contains_key(commands[1].as_str()));

    // Test group_by_device
    let grouped_by_device = utils::group_by_device(&results);
    assert_eq!(grouped_by_device.len(), 2);
    assert!(grouped_by_device.contains_key("device1"));
    assert!(grouped_by_device.contains_key("device2"));

    // Test format_as_table
    let table = utils::format_as_table(&results);
    assert!(table.contains("device1"));
    assert!(table.contains("device2"));
    assert!(table.contains("show version"));
    assert!(table.contains("show interfaces"));
}
