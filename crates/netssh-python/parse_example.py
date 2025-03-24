#!/usr/bin/env python3
"""
Example demonstrating how to connect to a device, run a command, and parse the output using TextFSM.
"""

import sys
import os
import json
import logging
from typing import Dict, List, Any, Optional

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(levelname)s: %(message)s')
logger = logging.getLogger(__name__)

# Add necessary paths
sys.path.append(os.path.dirname(os.path.abspath(__file__)))

# Import required modules
try:
    # Import netssh_rs modules - these will be available when the package is installed
    import netssh_rs
    from netssh_rs import PyDeviceConfig, PyNetworkDevice  # type: ignore
    from textfsm import NetworkOutputParser, parse_output  # type: ignore
except ImportError as e:
    logger.error(f"Error importing modules: {e}")
    logger.error("Make sure you've built the netssh_rs package: cargo build --release")
    sys.exit(1)

def using_native_parser() -> None:
    """Use the NetworkOutputParser directly with command output from netssh_rs."""
    
    # Device connection details
    device = {
        "device_type": "cisco_ios",
        "host": "192.168.1.25",  # Update with your device IP
        "username": "admin",     # Update with your username
        "password": "password",  # Update with your password
        "port": 22,
        "secret": "secret"       # Update with your enable secret if needed
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
    
    # Create NetworkOutputParser instance
    parser = NetworkOutputParser()
    logger.info(f"Template directory: {parser.template_dir}")
    
    # Command to execute
    command = "show interfaces description"
    
    # Connect to the device and execute command
    with PyNetworkDevice.create(config) as net_device:
        logger.info(f"Connecting to {device['host']}...")
        net_device.connect()
        
        # Run command and get output
        logger.info(f"Running '{command}'...")
        output = net_device.send_command(command)
        
        # Parse the output
        logger.info("Parsing output with TextFSM...")
        parsed_data = parser.parse_output(
            platform=device["device_type"],
            command=command,
            data=output
        )
        
        # Print results
        logger.info("Raw output:")
        print(output)
        logger.info("Parsed output:")
        print(json.dumps(parsed_data, indent=2))

def using_helper_function() -> None:
    """Use the parse_output helper function."""
    
    # Sample output (used if you don't have a device to connect to)
    sample_output = """
Interface                      Status         Protocol Description
Gi0/0                          up             up       
Gi0/1                          up             up       
Gi0/2                          admin down     down     
Gi0/3                          admin down     down     
"""
    
    # Parse the output using the helper function
    parsed_data = parse_output(
        platform="cisco_ios",
        command="show interfaces description",
        data=sample_output
    )
    
    # Print results
    logger.info("Sample output:")
    print(sample_output)
    logger.info("Parsed output:")
    print(json.dumps(parsed_data, indent=2))
    
    # Access individual fields
    if parsed_data:
        logger.info("Accessing individual fields:")
        for interface in parsed_data:
            status = interface.get("status", "unknown")
            name = interface.get("port", "unknown")
            print(f"Interface {name} is {status}")

def main() -> int:
    """Run the example."""
    
    logger.info("TextFSM Parsing Example")
    logger.info("-" * 50)
    
    # Uncomment to use live device
    # using_native_parser()
    
    # Use sample data with helper function
    using_helper_function()
    
    logger.info("-" * 50)
    logger.info("Example complete")
    
    return 0

if __name__ == "__main__":
    sys.exit(main()) 