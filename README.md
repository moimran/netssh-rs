# Netssh-rs

A Rust implementation of Netssh, providing SSH connection handling for network devices. This project aims to provide similar functionality to the Python Netmiko library while leveraging Rust's performance benefits and safety guarantees.

## Features

- SSH connection handling for various network device types
- Session logging
- Error handling with custom error types
- Thread-safe session management
- Async support (coming soon)

## Supported Device Types

- Cisco IOS
- Cisco IOS-XE
- Cisco NX-OS
- Cisco ASA
- Cisco IOS-XR
- Arista EOS
- Juniper JUNOS
- More coming soon...

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
netssh-rs = "0.1.0"
```

## Usage

Here's a basic example of connecting to a Cisco IOS device:

```rust
use netssh_rs::ConnectHandler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut connection = ConnectHandler::connect(
        "cisco_ios",
        "192.168.1.1".to_string(),
        "admin".to_string(),
        Some("password123".to_string()),
        None,
        Some(60),
    )?;

    connection.enable_session_log("session.log")?;
    connection.write_channel("show version\n")?;
    let output = connection.read_channel()?;
    println!("Command output:\n{}", output);

    Ok(())
}
```

## Running Examples

The repository includes several examples that demonstrate how to use netssh-rs. You can run them using Cargo:

```bash
# Run the basic connection example
cargo run --example basic_connection

# Test logging functionality
cargo run --example test_logging

# Connect to Cisco devices
cargo run --example cisco_xr
cargo run --example cisco_asa
cargo run --example cisco_devices
```

### Environment Variables

Most examples require setting environment variables for device credentials:

```bash
export DEVICE_HOST=192.168.1.1
export DEVICE_USER=admin
export DEVICE_PASS=password
export DEVICE_SECRET=enable_password  # For privileged mode on some devices
```

### Example Descriptions

- `basic_connection.rs` - Demonstrates a simple SSH connection to a device
- `test_logging.rs` - Shows how the logging functionality works
- `cisco_xr.rs` - Example for connecting to Cisco XR devices
- `cisco_asa.rs` - Example for connecting to Cisco ASA devices
- `cisco_devices.rs` - Tests connections to multiple Cisco device types

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
