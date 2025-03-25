#!/usr/bin/env python3
"""
Simple test file for VSCode IntelliSense with netssh_rs.
"""

from typing import Dict, Union, List
from netssh_rs import (
    PyDeviceConfig,
    PyNetworkDevice,
    PyDeviceInfo,
    PyCommandResult,
    PyParallelExecutionManager,
    initialize_logging
)

# Initialize logging
initialize_logging(debug=True, console=True)

# Create a device configuration
config = PyDeviceConfig(
    device_type="cisco_ios",
    host="192.168.1.1",
    username="admin",
    password="password123",
    port=22,
    secret="enable_secret"
)

# Create a network device
device = PyNetworkDevice.create(config)

# Connect to device
device.connect()

# Send a command
output = device.send_command("show version")
print(f"Output: {output}")

# Get device type
device_type = device.get_device_type()
print(f"Device type: {device_type}")

# Terminal settings
device.terminal_settings()
device.set_terminal_width(80)
device.disable_paging()

# Check config mode
is_config_mode = device.check_config_mode()
if not is_config_mode:
    device.enter_config_mode()
    device.exit_config_mode()

# Parallel execution
manager = PyParallelExecutionManager()
manager.set_max_concurrency(5)
manager.set_connection_timeout(30)
manager.set_command_timeout(60)
manager.set_failure_strategy("continue")

# Execute command
device_configs: Dict[PyDeviceConfig, Union[str, List[str]]] = {
    config: "show version"
}
results = manager.execute_commands(device_configs)

# Close connection
device.close()
manager.cleanup() 