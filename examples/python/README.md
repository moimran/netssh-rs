# netssh_rs Python Bindings

Python bindings for the netssh-rs Rust library, providing SSH connections to network devices.

## Installation

### From PyPI (not yet available)

```bash
pip install netssh_rs
```

### From Source

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/netssh-rs.git
   cd netssh-rs
   ```

2. Install maturin using uv:
   ```bash
   uv pip install maturin
   ```

3. Build and install the package:
   ```bash
   make setup
   make develop
   ```

   Or manually:
   ```bash
   maturin develop
   ```

## Usage

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
device = netssh_rs.PyNetworkDevice.create(config)

# Using context manager (recommended)
with device:
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

# Without context manager
device = netssh_rs.PyNetworkDevice.create(config)
try:
    device.connect()
    output = device.send_command("show ip interface brief")
    print(output)
finally:
    device.close()
```

## Supported Device Types

- cisco_ios
- cisco_nxos
- cisco_xr
- cisco_asa
- juniper_junos

## API Reference

### PyDeviceConfig

Configuration for connecting to a network device.

```python
config = netssh_rs.PyDeviceConfig(
    device_type="cisco_ios",  # Required: Device type
    host="192.168.1.1",       # Required: Hostname or IP
    username="admin",         # Required: Username
    password="password",      # Optional: Password
    port=22,                  # Optional: SSH port (default: 22)
    timeout_seconds=60,       # Optional: Timeout in seconds
    secret="enable_secret",   # Optional: Enable secret
    session_log="session.log" # Optional: Path to session log
)
```

### PyNetworkDevice

Main class for interacting with network devices.

```python
# Create device
device = netssh_rs.PyNetworkDevice.create(config)

# Connect to device
device.connect()

# Send command
output = device.send_command("show version")

# Enter configuration mode
device.enter_config_mode(None)  # or specify custom command

# Send configuration commands
device.send_command("hostname new-name")

# Exit configuration mode
device.exit_config_mode(None)  # or specify custom command

# Save configuration
device.save_configuration()

# Close connection
device.close()
```

## Error Handling

All methods can raise exceptions derived from Python's built-in exceptions. Handle them appropriately:

```python
try:
    device.connect()
    output = device.send_command("show version")
except RuntimeError as e:
    print(f"Error: {e}")
```

## Performance

These bindings provide significant performance improvements over pure Python implementations by leveraging Rust's speed and memory safety.