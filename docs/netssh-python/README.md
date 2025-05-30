# Netssh-rs Python Bindings

This package provides Python bindings for the Netssh-rs library, which implements SSH connection handling for network devices. It offers similar functionality to the Python Netmiko library while leveraging Rust's performance benefits and safety guarantees.

## Features

- Fast, concurrent SSH connections to network devices
- TextFSM parsing of command outputs
- Support for various network device types
- Parallel command execution
- Session logging
- Error handling and recovery

## Installation

```bash
pip install netssh_rs
```

## Basic Usage

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
    
    # Send command with TextFSM parsing
    parsed_output = device.send_command_with_parse("show version", parse=True)
    print(parsed_output)
```

## TextFSM Parsing

The library supports parsing command outputs using TextFSM templates. This allows for structured data extraction from CLI outputs:

```python
# Single command with parsing
parsed_data = device.send_command_with_parse("show version", parse=True)

# Working with structured data
for entry in parsed_data:
    print(f"Hostname: {entry.get('hostname')}")
    print(f"Version: {entry.get('version')}")
```

## Parallel Command Execution with Parsing

Execute commands on multiple devices in parallel and parse the outputs:

```python
# Create multiple device configurations
devices = [
    netssh_rs.PyDeviceConfig(device_type="cisco_ios", host="192.168.1.1", username="admin", password="password"),
    netssh_rs.PyDeviceConfig(device_type="cisco_ios", host="192.168.1.2", username="admin", password="password"),
]

# Create parallel execution manager
manager = netssh_rs.PyParallelExecutionManager(max_concurrency=5)

# Execute commands with parsing
commands = ["show version", "show interfaces"]
results = manager.execute_commands_on_all_with_parse(devices, commands, parse=True)

# Access parsed results
for result in results.get_command_results("show version"):
    device_id = result.device_id
    parsed_data = results.get_parsed_result(device_id, "show version")
    print(f"Device {device_id}: {parsed_data}")
```

## API Reference

### PyNetworkDevice

Methods for sending commands:

- `send_command(command: str)`: Send a command and return the raw output
- `send_command_with_parse(command: str, parse: bool)`: Send a command and optionally parse the output using TextFSM

### PyCommandResult

Methods for working with command results:

- `to_dict()`: Convert the result to a dictionary
- `parse_output()`: Parse the command output using TextFSM

### PyParallelExecutionManager

Methods for parallel command execution:

- `execute_command_on_all(devices, command)`: Execute a single command on multiple devices
- `execute_command_on_all_with_parse(devices, command, parse)`: Execute a command with parsing
- `execute_commands_on_all(devices, commands)`: Execute multiple commands on multiple devices
- `execute_commands_on_all_with_parse(devices, commands, parse)`: Execute commands with parsing
- `execute_commands(device_commands)`: Execute different commands on different devices
- `execute_commands_with_parse(device_commands, parse)`: Execute different commands with parsing

### PyBatchCommandResults

Methods for accessing parsed results:

- `get_parsed_result(device_id, command)`: Get parsed result for a specific device and command
- `get_parsed_outputs()`: Get all parsed outputs as a dictionary
- `get_command_results(command)`: Get results for a specific command
- `get_all_results()`: Get all command results

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

## Examples

See the `python/textfsm_example.py` file for detailed examples of using TextFSM parsing capabilities.

# netssh-rs Python Module with TextFSM Parsing

This module provides Python bindings for the netssh-rs Rust library, with integrated TextFSM parsing capabilities for network device command outputs.

## Features

- Fast and reliable SSH connections to network devices using Rust's netssh-rs library
- Automatic parsing of command outputs using TextFSM templates
- Support for parallel execution of commands across multiple devices
- Easy-to-use Python API with both object-oriented and functional interfaces

## Installation

```bash
# After building netssh-rs Rust library
cd /path/to/netssh-rs
pip install -e .
```

## Usage

### Basic Usage

```python
from netssh_parser import NetSSHParser, send_command

# Quick one-liner to send a command and parse the output
result = send_command(
    device_type="cisco_ios",
    host="192.168.1.1",
    username="admin",
    password="password",
    command="show ip interface brief",
    parse=True
)

print(result)  # List of dictionaries with parsed output

# Object-oriented approach
with NetSSHParser(
    device_type="cisco_ios",
    host="192.168.1.1",
    username="admin",
    password="password"
) as device:
    # Send command and parse output
    parsed_output = device.send_command("show ip interface brief")
    
    # Send command without parsing
    raw_output = device.send_command("show version", parse=False)
    
    # Send command and get both raw and parsed output with metadata
    result = device.send_command_with_metadata("show ip route")
```

### Parallel Execution

```python
from netssh_parser import ParallelParser

# Create a parallel parser
parser = ParallelParser(
    max_concurrency=10,
    command_timeout_seconds=60,
    failure_strategy="continue"
)

# Define devices
devices = [
    {
        "device_type": "cisco_ios",
        "host": "192.168.1.1",
        "username": "admin",
        "password": "password"
    },
    {
        "device_type": "juniper_junos",
        "host": "192.168.1.2",
        "username": "admin",
        "password": "password"
    }
]

# Execute command on all devices with parsing
results = parser.execute_command(
    devices=devices,
    command="show version",
    parse=True
)

# Access parsed results
parsed_results = results['parsed']
raw_results = results['raw']
```

### Command-Line Interface

The module also provides a simple command-line interface:

```bash
python -m netssh_parser --device_type cisco_ios --host 192.168.1.1 --username admin --password password --command "show ip interface brief"
```

## TextFSM Templates

This module uses TextFSM templates to parse command outputs from network devices. The templates are located in the `textfsm/templates` directory, and the mapping between commands and templates is defined in the `textfsm/templates/index` file.

### Adding New Templates

1. Create a new TextFSM template file in the `textfsm/templates` directory
2. Add an entry to the `textfsm/templates/index` file with the format:
   ```
   template_file.textfsm, .*, platform_pattern, command_pattern
   ```

## API Reference

### NetSSHParser

- `__init__(device_type, host, username, password=None, port=None, timeout_seconds=None, secret=None, session_log=None, debug=False)`
- `connect()`
- `close()`
- `send_command(command, parse=True)`
- `send_command_with_metadata(command)`

### ParallelParser

- `__init__(max_concurrency=None, command_timeout_seconds=None, connection_timeout_seconds=None, failure_strategy="continue", reuse_connections=True, debug=False)`
- `execute_command(devices, command, parse=True)`

### Functions

- `send_command(device_type, host, username, command, password=None, port=None, timeout_seconds=None, secret=None, session_log=None, parse=True, debug=False)`
- `connect(device_type, host, username, password=None, port=None, timeout_seconds=None, secret=None, session_log=None, debug=False)` (context manager)

## Integration with TextFSM

The module integrates with TextFSM through the `textfsm/parse_output.py` module, which provides functions for parsing command outputs using TextFSM templates:

- `parse_output(device_type, command, output)`: Parses command output using TextFSM and returns a list of dictionaries
- `parse_output_to_dict(device_type, command, output)`: Returns a dictionary with both parsed data and metadata
- `parse_raw_output(raw_output, template_file)`: Parses output using a specific template file

## License

This module is released under the same license as netssh-rs. 