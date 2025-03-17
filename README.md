# Netssh-rs

A Rust implementation of Netssh, providing SSH connection handling for network devices. This project aims to provide similar functionality to the Python Netmiko library while leveraging Rust's performance benefits and safety guarantees.

## Features

- SSH connection handling for various network device types
- REST API for device management and command execution
- Standardized JSON responses for network operations
- Session logging
- Error handling with custom error types
- Thread-safe session management
- Concurrent connection handling
- Async support
- **Python bindings** for using netssh-rs from Python code

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

## REST API

Netssh-rs includes a REST API server that allows you to execute commands on network devices via HTTP requests. The API provides standardized JSON responses for all operations.

### Starting the API Server

```rust
use actix_web::{web, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Start API server
    HttpServer::new(move || {
        App::new()
            .configure(netssh_rs::api::routes::configure)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

### API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | /api/execute/show | Execute a show command |
| POST | /api/execute/configure | Execute configuration commands |
| POST | /api/interfaces/configure | Configure an interface |
| GET | /health | Health check endpoint |

### Example Requests

#### Execute a Show Command

```bash
curl -X POST http://localhost:8080/api/execute/show \
  -H "Content-Type: application/json" \
  -d '{
    "device": {
      "device_type": "cisco_ios",
      "host": "192.168.1.1",
      "username": "admin",
      "password": "cisco123",
      "port": 22,
      "timeout": 60,
      "secret": "enable_secret"
    },
    "command": "show ip interface brief"
  }'
```

#### Execute Configuration Commands

```bash
curl -X POST http://localhost:8080/api/execute/configure \
  -H "Content-Type: application/json" \
  -d '{
    "device": {
      "device_type": "cisco_ios",
      "host": "192.168.1.1",
      "username": "admin",
      "password": "cisco123",
      "port": 22,
      "timeout": 60,
      "secret": "enable_secret"
    },
    "commands": [
      "interface GigabitEthernet0/1",
      "description WAN Connection",
      "ip address 10.0.0.1 255.255.255.0",
      "no shutdown"
    ]
  }'
```

#### Configure an Interface

```bash
curl -X POST http://localhost:8080/api/interfaces/configure \
  -H "Content-Type: application/json" \
  -d '{
    "device": {
      "device_type": "cisco_ios",
      "host": "192.168.1.1",
      "username": "admin",
      "password": "cisco123",
      "port": 22,
      "timeout": 60,
      "secret": "enable_secret"
    },
    "name": "GigabitEthernet0/1",
    "description": "WAN Connection",
    "ip_address": "10.0.0.1",
    "subnet_mask": "255.255.255.0",
    "admin_status": "up"
  }'
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
cargo run --example cisco_ios

# Start the REST API server
cargo run --example rest_api_example
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
- `cisco_ios.rs` - Example for connecting to Cisco IOS devices
- `rest_api_example.rs` - Starts a REST API server with pre-configured devices

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
## Python Bindings

Netssh-rs includes Python bindings that allow you to use the library from Python code. This provides the performance benefits of Rust with the convenience of Python.

### Installation

```bash
# Install from source
uv pip install maturin
cd netssh-rs
make setup
make develop
```

### Python Usage

```python
import netssh_rs

# Initialize logging
netssh_rs.initialize_logging(debug=True, console=True)

# Create a device configuration
config = netssh_rs.PyDeviceConfig(
    device_type="cisco_ios",
    host="192.168.1.1",
    username="admin",
    password="password",
    port=22,
    timeout_seconds=60,
    secret="enable_secret",
    session_log="logs/device_session.log"
)

# Create and connect to a device
with netssh_rs.PyNetworkDevice.create(config) as device:
    device.connect()
    
    # Send commands
    output = device.send_command("show version")
    print(output)
    
    # Configure device
    device.enter_config_mode(None)
    device.send_command("interface GigabitEthernet1")
    device.send_command("description Configured by Python")
    device.exit_config_mode(None)
    
    # Save configuration
    device.save_configuration()
```

For more details, see the [Python README](python/README.md).

## License

This project is licensed under the MIT License - see the LICENSE file for details.
This project is licensed under the MIT License - see the LICENSE file for details.
