# Netmiko-rs

A Rust implementation of Netmiko, providing SSH connection handling for network devices. This project aims to provide similar functionality to the Python Netmiko library while leveraging Rust's performance benefits and safety guarantees.

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
netmiko-rs = "0.1.0"
```

## Usage

Here's a basic example of connecting to a Cisco IOS device:

```rust
use netmiko_rs::ConnectHandler;

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

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
