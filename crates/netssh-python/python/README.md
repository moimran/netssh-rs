# Netssh-rs Python Bindings

This package provides Python bindings for the Netssh-rs library, which implements SSH connection handling for network devices. It offers similar functionality to the Python Netmiko library while leveraging Rust's performance benefits and safety guarantees.

## Installation

```bash
pip install netssh_rs
```

## Usage

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

## Supported Device Types

- Cisco IOS
- Cisco IOS-XE
- Cisco NX-OS
- Cisco ASA
- Cisco IOS-XR
- Arista EOS
- Juniper JUNOS
- More coming soon...

## API Reference

### PyDeviceConfig

Configuration for connecting to a network device.

- `device_type`: String - Type of device (e.g., "cisco_ios", "juniper_junos")
- `host`: String - Hostname or IP address of the device
- `username`: String - Username for authentication
- `password`: Optional[String] - Password for authentication
- `port`: Optional[int] - SSH port (default: 22)
- `timeout_seconds`: Optional[int] - Connection timeout in seconds
- `secret`: Optional[String] - Enable secret for privileged mode
- `session_log`: Optional[String] - Path to session log file

### PyNetworkDevice

Represents a connection to a network device.

Methods:
- `connect()`: Connect to the device
- `close()`: Close the connection
- `send_command(command: str)`: Send a command and return the output
- `check_config_mode()`: Check if in configuration mode
- `enter_config_mode(config_command: Optional[str])`: Enter configuration mode
- `exit_config_mode(exit_command: Optional[str])`: Exit configuration mode
- `save_configuration()`: Save or commit the configuration 