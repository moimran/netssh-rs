# Netssh-rs

A Rust implementation of Netssh, providing SSH connection handling for network devices. This project aims to provide similar functionality to the Python Netmiko library while leveraging Rust's performance benefits and safety guarantees.

## Project Structure

Netssh-rs is organized as a Rust workspace with multiple specialized crates:

### Core Crates
- **netssh-core** - Core SSH functionality for network devices
- **netssh-python** - Python bindings for netssh-rs
- **netssh-textfsm** - TextFSM parsing functionality for network command output
- **scheduler** - Job scheduling system for network automation tasks
- **shared-config** - Unified configuration management across all crates

### Workspace Organization
```
netssh-rs/
├── crates/           # Individual crate implementations
├── docs/             # Centralized documentation for all crates
├── examples/         # Centralized examples organized by crate
├── config.toml       # Unified workspace configuration
└── README.md         # This file
```

This modular structure allows you to build and use each component independently while maintaining shared configuration and documentation.

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

## Building

You can build each component separately using the provided Makefile:

```bash
# Build everything
make all

# Build only the core library
make build-core

# Build only the REST API
make build-api

# Build only the Python bindings
make build-python
```

## Usage

### Rust Library

Add this to your `Cargo.toml`:

```toml
[dependencies]
netssh-core = "0.1.0"
```

Here's a basic example of connecting to a Cisco IOS device:

```rust
use netssh_core::DeviceConfig;
use netssh_core::DeviceFactory;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = DeviceConfig {
        device_type: "cisco_ios".to_string(),
        host: "192.168.1.1".to_string(),
        username: "admin".to_string(),
        password: Some("password123".to_string()),
        port: Some(22),
        timeout: Some(std::time::Duration::from_secs(60)),
        secret: None,
        session_log: Some("session.log".to_string()),
    };

    let mut device = DeviceFactory::create_device(&config)?;
    device.connect()?;
    
    let output = device.send_command("show version")?;
    println!("Command output:\n{}", output);

    device.close()?;
    Ok(())
}
```

### REST API

To use the REST API component, build and run the API server:

```bash
# Build and run the API server
make run-api
```

#### API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | /api/execute/show | Execute a show command |
| POST | /api/execute/configure | Execute configuration commands |
| POST | /api/interfaces/configure | Configure an interface |
| GET | /health | Health check endpoint |

### Python Bindings

Install the Python bindings:

```bash
# Install in development mode
make develop-python

# Or build and install the wheel
make install-python
```

Then use it in your Python code:

```python
import netssh_rs

# Initialize logging
netssh_rs.initialize_logging(level="debug", log_to_file=True, log_file_path="logs/netssh-rs.log")

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
```

## Running Examples

The repository includes several examples organized by crate in the centralized `examples/` directory:

### Shared Configuration Examples
```bash
# Demonstrate unified configuration system
cargo run --example unified_config_demo -p shared-config

# Show cross-crate configuration integration
cargo run --example cross_crate_integration_demo -p shared-config
```

### Scheduler Examples
```bash
# Basic scheduler usage
cargo run --example basic_usage -p scheduler

# Logging demonstration
cargo run --example logging_demo -p scheduler

# Connection reuse optimization
cargo run --example phase1_connection_reuse -p scheduler

# Web board interface
cargo run --example with_board -p scheduler
```

### Running the Scheduler Service
```bash
# Start the scheduler with job processing
cargo run -p scheduler
```

### Environment Variables

Most examples require setting environment variables for device credentials:

```bash
export DEVICE_HOST=192.168.1.1
export DEVICE_USER=admin
export DEVICE_PASS=password
export DEVICE_SECRET=enable_password  # For privileged mode on some devices
```

## Documentation

Comprehensive documentation for each crate is available in the `docs/` directory:

- **[netssh-core](docs/netssh-core/README.md)** - Core SSH functionality and settings migration
- **[netssh-python](docs/netssh-python/README.md)** - Python bindings and TextFSM examples
- **[netssh-textfsm](docs/netssh-textfsm/README.md)** - TextFSM parsing functionality
- **[scheduler](docs/scheduler/README.md)** - Job scheduling system and architecture
- **[shared-config](docs/shared-config/README.md)** - Unified configuration system
- **[Project Documentation](docs/project/)** - General project context and organization

### Key Documentation
- [Workspace Reorganization](docs/project/workspace_reorganization.md) - Details about the new project structure
- [Settings Migration Guide](docs/netssh-core/netssh_settings_migration.md) - Migration from individual to unified configuration
- [Thread Safety Migration](docs/netssh-core/thread_safety_migration_notes.md) - PyO3 compatibility and thread safety
- [SSH Connection Scaling](docs/scheduler/ssh_connection_scaling_architecture.md) - Performance optimization strategies

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
