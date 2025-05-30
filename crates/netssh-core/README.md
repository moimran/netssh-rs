# netssh-core

Core SSH functionality for netssh-rs - A Rust library for SSH connections to network devices.

## Improvements Overview

This version of netssh-core includes several significant improvements:

### Memory Management

- **Buffer Pool**: Implemented a thread-safe buffer pool that reuses byte buffers across operations, reducing memory allocations and improving performance. The pool automatically manages buffer lifecycle and handles buffer sizing based on requirements.

- **Binary Data Handling**: Replaced String usage with Vec<u8> for binary data paths, avoiding unnecessary UTF-8 encoding/decoding operations. This provides better performance and correctness when handling raw network data.

### Concurrency

- **Timeout Semaphore**: Added a semaphore implementation with timeout handling for connection limits, preventing excessive queuing during high load. The semaphore provides a reliable way to limit concurrent connections while ensuring timely responses.

- **Async/Await Patterns**: Ensured all I/O operations in async contexts use truly non-blocking calls, improving performance in high-load scenarios.

### Configuration

- **Settings System**: Implemented a flexible settings system that supports JSON configuration files, allowing easy customization of all timeout values and other parameters. The settings system includes comprehensive documentation of all parameters.

- **Global Settings**: Added a global settings mechanism that ensures consistent configuration across the application, with thread-safe access.

### Testing

- **Mock Device**: Created a comprehensive mock network device implementation for testing, allowing simulation of various device types and behaviors.

- **Unit Tests**: Added extensive unit tests for the buffer pool, semaphore, and settings implementations.

- **Integration Tests**: Added integration tests that verify the proper functioning of the library with the mock device.

## Parallel Command Execution

The `netssh-core` library includes a parallel command execution system that allows for efficient execution of commands across multiple network devices simultaneously. This functionality is particularly useful for bulk operations in network automation scenarios.

### Features

- Execute the same command on multiple devices in parallel
- Execute multiple commands sequentially on all devices in parallel
- Execute different commands on different devices
- Control concurrency with configurable limits
- Handle timeouts and failures with customizable strategies
- Collect standardized results with detailed information
- Reuse connections for sequential commands to improve performance

### Usage (Rust)

```rust
use netssh_core::{DeviceConfig, ParallelExecutionManager, ParallelExecutionConfig, FailureStrategy};
use std::time::Duration;

// Create device configurations
let devices = vec![
    DeviceConfig {
        device_type: "cisco_ios".to_string(),
        host: "192.168.1.1".to_string(),
        username: "admin".to_string(),
        password: Some("password".to_string()),
        // ... other fields
    },
    // ... more devices
];

// Create a parallel execution manager with custom configuration
let config = ParallelExecutionConfig {
    max_concurrency: 10,
    command_timeout: Some(Duration::from_secs(60)),
    connection_timeout: Some(Duration::from_secs(30)),
    failure_strategy: FailureStrategy::ContinueDevice,
    reuse_connections: true,
};

let mut manager = ParallelExecutionManager::with_config(config);

// Execute the same command on all devices in parallel
let command = "show version".to_string();
let results = manager.execute_command_on_all(devices, command).await?;

// Process results
println!("Device count: {}", results.device_count);
println!("Command count: {}", results.command_count);
println!("Success count: {}", results.success_count);
println!("Failure count: {}", results.failure_count);

// Get results for a specific device
if let Some(device_results) = results.get_device_results("192.168.1.1") {
    for result in device_results {
        println!("Command: {}, Status: {:?}", result.command, result.status);
        println!("Output: {}", result.output);
    }
}
```

### Result Processing

The library includes utilities for processing and formatting command results:

```rust
use netssh_core::parallel_execution::utils;

// Format results as table
let table = utils::format_as_table(&results);
println!("{}", table);

// Export to JSON
let json = utils::to_json(&results)?;

// Export to CSV
let csv = utils::to_csv(&results);

// Group results by command
let grouped_by_command = utils::group_by_command(&results);

// Group results by device
let grouped_by_device = utils::group_by_device(&results);

// Compare outputs for the same command across devices
let output_comparison = utils::compare_outputs(&results, "show version");
```

### Failure Strategies

The library supports three failure strategies:

- `ContinueDevice`: Continue executing remaining commands for a device even if a command fails
- `StopDevice`: Skip remaining commands for a device if any command fails for that device
- `StopAll`: Stop all execution across all devices if any command fails on any device

### Python Bindings

The parallel execution functionality is also exposed through Python bindings. See the Python documentation for details.

## Device Type Autodetection

The `netssh-core` library now includes a powerful autodetection mechanism that can automatically determine the device type based on command outputs and pattern matching. This is particularly useful when connecting to various network devices without knowing their specific types.

### How to Use Autodetection

To use autodetection, simply set the `device_type` to "autodetect" in your `DeviceConfig`:

```rust
use netssh_core::{DeviceConfig, DeviceFactory};

// Create a device configuration with autodetect
let config = DeviceConfig {
    device_type: "autodetect".to_string(),
    host: "192.168.1.1".to_string(),
    username: "admin".to_string(),
    password: Some("password".to_string()),
    port: None,
    timeout: None,
    secret: None,
    session_log: None,
};

// Create a device connection - autodetection happens automatically
let device = DeviceFactory::create_device(&config)?;

// Now you can interact with the device normally
device.connect()?;
let output = device.send_command("show version")?;
```

### How Autodetection Works

When `device_type` is set to "autodetect", the library:

1. Establishes an SSH connection to the device
2. Sends various identification commands (like "show version")
3. Analyzes the output with regular expressions to identify the device
4. Returns the appropriate device type (e.g., "cisco_ios", "juniper_junos", etc.)
5. Creates a new connection with the detected device type

### Supported Device Types for Autodetection

The following device types can be automatically detected:

- Alcatel AOS (`alcatel_aos`)
- Alcatel SROS (`alcatel_sros`)
- Allied Telesis AW+ (`allied_telesis_awplus`)
- Apresia AEOS (`apresia_aeos`)
- Arista EOS (`arista_eos`)
- Aruba AOSCX (`aruba_aoscx`)
- Ciena SAOS (`ciena_saos`)
- Cisco ASA (`cisco_asa`)
- Cisco FTD (`cisco_ftd`)
- Cisco IOS (`cisco_ios`)
- Cisco IOS XE (`cisco_xe`)
- Cisco IOS XR (`cisco_xr`)
- Cisco NX-OS (`cisco_nxos`)
- Cisco Viptela (`cisco_viptela`)
- Cisco WLC (`cisco_wlc`)
- Dell Force10 (`dell_force10`)
- Dell OS9 (`dell_os9`)
- Dell OS10 (`dell_os10`)
- Dell PowerConnect (`dell_powerconnect`)
- Ericsson IPOS (`ericsson_ipos`)
- Extreme EXOS (`extreme_exos`)
- Extreme NetIron (`extreme_netiron`)
- Extreme SLX (`extreme_slx`)
- Extreme Tierra (`extreme_tierra`)
- F5 Linux (`f5_linux`)
- F5 TMSH (`f5_tmsh`)
- FlexVNF (`flexvnf`)
- Fortinet (`fortinet`)
- HP Comware (`hp_comware`)
- HP ProCurve (`hp_procurve`)
- Huawei (`huawei`)
- Huawei SmartAX (`huawei_smartax`)
- Juniper JunOS (`juniper_junos`)
- Linux (`linux`)
- Mellanox MLNXOS (`mellanox_mlnxos`)
- Netgear ProSAFE (`netgear_prosafe`)
- OneAccess OneOS (`oneaccess_oneos`)
- Palo Alto PanOS (`paloalto_panos`)
- Supermicro SMIS (`supermicro_smis`)
- Ubiquiti EdgeSwitch (`ubiquiti_edgeswitch`)
- Yamaha (`yamaha`)

### Advanced Usage: Direct Access to SSHDetect

For more fine-grained control, you can use the `SSHDetect` class directly:

```rust
use netssh_core::{DeviceConfig, SSHDetect};

// Create a device configuration for autodetection
let config = DeviceConfig {
    device_type: "autodetect".to_string(),
    host: "192.168.1.1".to_string(),
    username: "admin".to_string(),
    password: Some("password".to_string()),
    // ... other fields
};

// Create an SSHDetect instance
let mut detector = SSHDetect::new(&config)?;

// Run the autodetection
match detector.autodetect()? {
    Some(device_type) => println!("Detected device type: {}", device_type),
    None => println!("Could not detect device type"),
}

// Close the connection
detector.disconnect()?;
```

## Using the Settings System

The settings system allows you to customize all aspects of netssh-rs behavior through the shared workspace configuration. Here's how to use it:

```rust
// Initialize with shared workspace configuration (recommended)
Settings::init_from_workspace_config().expect("Failed to initialize settings");

// Or initialize with default settings
Settings::init(None).expect("Failed to initialize settings");

// Or initialize with a custom settings file (legacy)
Settings::init(Some("settings.json")).expect("Failed to initialize settings");

// Get settings values using the helper functions
let timeout = get_network_timeout(NetworkTimeoutType::TcpConnect);
let buffer_size = get_buffer_setting(BufferSettingType::ReadBufferSize);

// Update settings programmatically
Settings::update(|s| {
    s.network.tcp_connect_timeout_secs = 120;
    s.buffer.read_buffer_size = 131072;
}).expect("Failed to update settings");
```

## Buffer Pool Usage

The buffer pool provides efficient memory reuse for I/O operations:

```rust
// Get the global buffer pool
let pool = BufferPool::global();

// Get a buffer with at least 4KB capacity
let mut buffer = pool.get_buffer(4096);

// Use the buffer
buffer.resize(100, 0);
some_reader.read(&mut buffer[..]).expect("Read failed");

// Convert to string if needed
let text = buffer.as_utf8_lossy_string();
println!("Read: {}", text);

// Buffer is automatically returned to the pool when dropped
```

## Semaphore Usage

The timeout semaphore provides controlled access to limited resources:

```rust
// Create a semaphore with 10 permits
let semaphore = TimeoutSemaphore::new(10);

// Try to acquire a permit without waiting
if let Ok(permit) = semaphore.try_acquire() {
    // Use the resource
    // Permit is automatically released when dropped
} else {
    println!("No permits available");
}

// Acquire with timeout
match semaphore.acquire_timeout(Some(Duration::from_secs(5))) {
    Ok(permit) => {
        // Use the resource
        // Permit is automatically released when dropped
    }
    Err(SemaphoreError::Timeout) => {
        println!("Timed out waiting for permit");
    }
    Err(e) => {
        println!("Error: {:?}", e);
    }
}
```

## Testing with the Mock Device

The mock device allows testing without real network devices:

```rust
// Create and configure a mock device
let mut device = MockNetworkDevice::new();
device.set_hostname("Router1")
      .set_prompt_style(PromptStyle::CiscoIOS)
      .add_auth_credentials("admin", "password")
      .add_command_response("show version", "IOS Version 15.2");

// Start the mock server
device.start().expect("Failed to start mock device");

// Use the device in tests
let port = device.port();
let mut conn = BaseConnection::new().expect("Failed to create connection");
conn.connect("127.0.0.1", "admin", Some("password"), Some(port), None)
    .expect("Connection failed");

// Send commands to the mock device
let response = conn.send_command("show version").expect("Command failed");
assert!(response.contains("IOS Version 15.2"));
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. 