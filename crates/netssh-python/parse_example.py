#!/usr/bin/env python3
"""
Example demonstrating how to connect to a network device and execute commands.
"""

import sys
import os
import json
import logging
from typing import Dict, List, Any, Optional

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(levelname)s: %(message)s')
logger = logging.getLogger(__name__)

# Remove the current directory from path to avoid using local netssh_rs package
current_dir = os.path.dirname(os.path.abspath(__file__))
if current_dir in sys.path:
    sys.path.remove(current_dir)
    logger.info(f"Removed {current_dir} from sys.path")

for p in sys.path:
    logger.info(f"Path: {p}")

# Import required modules
try:
    from netssh_rs import PyDeviceConfig, PyNetworkDevice  # type: ignore
    logger.info("Successfully imported from netssh_rs")
except ImportError as e:
    logger.error(f"Error importing modules: {e}")
    logger.error("Make sure you've built the netssh_rs package: cargo build --release")
    sys.exit(1)

def device_example() -> None:
    """Connect to a device and run commands."""
    
    # Device connection details
    device = {
        "device_type": "cisco_ios",
        "host": "192.168.1.25",  # Update with your device IP
        "username": "admin",     # Update with your username
        "password": "moimran@123",  # Update with your password
        "port": 22,
        "secret": "moimran@123"       # Update with your enable secret if needed
    }
    
    # Create device configuration
    config = PyDeviceConfig(
        device_type=device["device_type"],
        host=device["host"],
        username=device["username"],
        password=device["password"],
        port=device["port"],
        secret=device["secret"]
    )
    
    # Command to execute
    command = "show interfaces description"
    
    # Connect to the device and execute command
    try:
        # Create the device object
        net_device = PyNetworkDevice.create(config)  # type: ignore
        
        logger.info(f"Connecting to {device['host']}...")
        net_device.connect()
        
        # Run command and get output
        logger.info(f"Running '{command}'...")
        result = net_device.send_command(command)  # type: ignore
        
        # Print results
        logger.info("Command output:")
        print(result)  # type: ignore
        
    
    finally:
        # Clean up the device connection
        if 'net_device' in locals():
            logger.info("Disconnecting...")
            net_device.close()  # type: ignore

def main() -> int:
    """Run the example."""
    
    logger.info("NetSSH-RS Python Example")
    logger.info("-" * 50)
    
    # Use the device example
    device_example()
    
    logger.info("-" * 50)
    logger.info("Example complete")
    
    return 0

if __name__ == "__main__":
    sys.exit(main()) 