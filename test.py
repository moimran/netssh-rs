#!/usr/bin/env python3
"""
External test file for netssh_rs module.
This file should be placed outside the workspace directory.
"""

# Import the netssh_rs module
try:
    from netssh_rs import PyDeviceConfig, PyNetworkDevice, initialize_logging, PyParallelExecutionManager, PyCommandResult, PyBatchCommandResults, PyDeviceInfo
    print("Successfully imported from netssh_rs")
except ImportError as e:
    print(f"Error importing netssh_rs: {e}")
    exit(1)

def main():
    """Test basic functionality."""
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
    
    print(f"Created device config for {config.device_type} device at {config.host}")
    
    # The rest would be actual device interaction, but we'll skip it
    # since this is just to test the imports
    print("External test successful!")

if __name__ == "__main__":
    main() 