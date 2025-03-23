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

## Parallel Command Execution

The netssh-rs Python bindings provide a powerful parallel command execution system that enables running commands on multiple devices simultaneously.

### Features

- Run the same command on multiple devices in parallel
- Execute multiple commands sequentially on all devices in parallel
- Execute different commands on different devices
- Control concurrency with configurable limits
- Handle timeouts and failures with customizable strategies
- Collect and process standardized results

### Basic Usage

```python
import netssh_rs

# Create device configurations
devices = [
    netssh_rs.PyDeviceConfig(
        device_type="cisco_ios",
        host="192.168.1.1",
        username="admin",
        password="cisco",
        port=22,
        timeout_seconds=30,
        secret="enable_secret",
        session_log=None
    ),
    netssh_rs.PyDeviceConfig(
        device_type="juniper_junos",
        host="192.168.1.2",
        username="admin",
        password="juniper",
        port=22,
        timeout_seconds=30,
        secret=None,
        session_log=None
    )
]

# Create a parallel execution manager
manager = netssh_rs.PyParallelExecutionManager(
    max_concurrency=10,
    command_timeout_seconds=60,
    connection_timeout_seconds=30,
    failure_strategy="continue_device",
    reuse_connections=True
)

# Execute the same command on all devices
results = manager.execute_command_on_all(devices, "show version")

# Process the results
print(f"Devices: {results.device_count}")
print(f"Commands: {results.command_count}")
print(f"Success: {results.success_count}, Failed: {results.failure_count}")

# Print results in table format
print(results.format_as_table())
```

### Multiple Commands

Execute multiple commands sequentially on all devices in parallel:

```python
commands = [
    "show version",
    "show interfaces brief",
    "show ip route summary"
]

results = manager.execute_commands_on_all(devices, commands)
```

### Device-Specific Commands

Execute different commands on different devices:

```python
device_commands = {
    devices[0]: ["show version", "show interfaces"],
    devices[1]: ["show system information", "show interfaces terse"]
}

results = manager.execute_commands(device_commands)
```

### Working with Results

The result object provides multiple ways to process and analyze the command execution results:

```python
# Get all results for a specific device
device_results = results.get_device_results("192.168.1.1")
for result in device_results:
    print(f"Command: {result.command}, Status: {result.status}")
    print(f"Output: {result.output}")

# Get results for a specific command across all devices
cmd_results = results.get_command_results("show version")

# Get successful/failed results
successful = results.get_successful_results()
failed = results.get_failed_results()

# Export results to different formats
json_data = results.to_json()
csv_data = results.to_csv()

# Compare outputs across devices for the same command
comparison = results.compare_outputs("show version")
for output, devices in comparison.items():
    print(f"Devices with identical output: {', '.join(devices)}")
```

### Failure Strategies

The manager supports different strategies for handling command failures:

- `continue_device`: Continue executing remaining commands for a device even if a command fails
- `stop_device`: Skip remaining commands for a device if any command fails for that device
- `stop_all`: Stop all execution across all devices if any command fails on any device

```python
# Change the failure strategy
manager.set_failure_strategy("stop_device")
```

### Context Manager Support

The parallel execution manager can be used as a context manager to ensure proper cleanup:

```python
with netssh_rs.PyParallelExecutionManager() as manager:
    results = manager.execute_command_on_all(devices, "show version")
    # Process results
# Connections are automatically cleaned up
```

For a complete example, see the `parallel_example.py` file in the Python examples directory.

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