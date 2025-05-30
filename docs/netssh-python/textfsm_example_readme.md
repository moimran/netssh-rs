# TextFSM Integration with netssh-rs

This README provides information on how to use TextFSM parsing functionality with netssh-rs to parse command outputs from network devices into structured data.

## Overview

TextFSM is a template-based parsing system that allows you to convert unstructured text output (such as the output of network device commands) into structured data formats like JSON or Python dictionaries. The netssh-rs Python package includes built-in TextFSM integration to parse command outputs from network devices.

## No External Dependencies

The TextFSM module is **included directly in the netssh-rs package**. You do not need to install TextFSM separately. This provides several benefits:

1. Simplified installation - one package includes everything
2. Guaranteed compatibility between netssh-rs and TextFSM
3. Ability to use TextFSM in air-gapped environments

## Installation

To use TextFSM parsing with netssh-rs, simply install the netssh-rs Python package:

```bash
pip install netssh-rs
```

The TextFSM module is included with the package and doesn't need to be installed separately.

## Available Templates

The package comes with a set of pre-built templates for various network devices and commands, including:
- Cisco IOS
- Cisco NXOS
- Arista EOS
- Juniper Junos
- And more

For Cisco IOS "show version" specifically, the template extracts:
- Software version
- Release
- Hostname
- Uptime
- Hardware models
- Serial numbers
- MAC addresses
- And more

## Basic Usage

Here's a simple example of how to use TextFSM with netssh-rs:

```python
# Import both device connectivity and TextFSM parsing from netssh_rs
from netssh_rs import (
    PyDeviceConfig, 
    PyNetworkDevice, 
    parse_output, 
    parse_output_to_json
)

# Connect to a device
config = PyDeviceConfig(
    device_type="cisco_ios",
    host="192.168.1.1",
    username="admin",
    password="password"
)
device = PyNetworkDevice.create(config)
device.connect()

# Send a command
result = device.send_command("show version")

if result.is_success():
    # Parse the output using TextFSM
    parsed_data = parse_output("cisco_ios", "show version", result.output)
    
    # Print the parsed data
    print(parsed_data)
    
    # Or convert to JSON
    json_data = parse_output_to_json("cisco_ios", "show version", result.output)
    print(json_data)
```

## Using the Example Script

The `example_parse_show_version.py` script demonstrates how to parse Cisco IOS "show version" command output using TextFSM:

1. Run the example with your device information:
```bash
python example_parse_show_version.py
```

2. The script will connect to your device, retrieve the "show version" output, and parse it into structured data.

## Parsing Existing Output (Without Device Connection)

If you already have command output and want to parse it without connecting to a device:

```python
from netssh_rs import parse_output, NetworkOutputParser

# Option 1: Using the parse_output function directly
with open("show_version_output.txt", "r") as f:
    output = f.read()

parsed_data = parse_output("cisco_ios", "show version", output)
print(parsed_data)

# Option 2: Using the NetworkOutputParser class
parser = NetworkOutputParser()
parsed_data = parser.parse_output("cisco_ios", "show version", output)
print(parsed_data)
```

## Creating Custom Templates

If you need to parse commands that don't have existing templates, you can create your own:

1. Create a template file using TextFSM syntax
2. Place it in your custom template directory
3. Use the NetworkOutputParser with a custom template directory

Example custom parser:
```python
from netssh_rs import NetworkOutputParser

# Create parser with custom template directory
parser = NetworkOutputParser(template_dir="/path/to/your/templates")

# Parse output
parsed_data = parser.parse_output("cisco_ios", "show version", output)
```

## Advanced Usage

### Custom Template Directory

You can specify a custom template directory:

```python
from netssh_rs import NetworkOutputParser

# Create parser with custom template directory
parser = NetworkOutputParser(template_dir="/path/to/your/templates")

# Parse output
parsed_data = parser.parse_output("cisco_ios", "show version", output)
```

### Error Handling

Always check for parser errors:

```python
parsed_data = parse_output("cisco_ios", "show version", output)
if parsed_data is None:
    print("Parsing failed - check that the template exists and matches the output format")
```

## Supported Command Reference

For Cisco IOS, the following commands have built-in TextFSM templates (among others):
- show version
- show interfaces
- show ip interface brief
- show running-config
- show cdp neighbors
- show inventory
- show ip route

For a complete list of supported commands and platforms, see the templates directory in the netssh-rs package. 