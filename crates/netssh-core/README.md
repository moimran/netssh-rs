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

## Using the Settings System

The settings system allows you to customize all aspects of netssh-rs behavior through a JSON configuration file. Here's how to use it:

```rust
// Initialize with default settings
Settings::init(None).expect("Failed to initialize settings");

// Or initialize with a custom settings file
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