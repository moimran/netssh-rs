#!/usr/bin/env python3
"""
Example demonstrating how to connect to a network device and execute commands.
"""

import sys
import logging
from typing import Dict, Any

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(levelname)s: %(message)s')
logger = logging.getLogger(__name__)

# Import required modules
try:
    # Try direct import first
    from netssh_rs import PyDeviceConfig, PyNetworkDevice, initialize_logging
    logger.info("Successfully imported from netssh_rs")
except ImportError as e:
    logger.error(f"Error importing from netssh_rs: {e}")
    
    try:
        # Fall back to nested import
        from netssh_rs.netssh_rs import PyDeviceConfig, PyNetworkDevice, initialize_logging
        logger.info("Successfully imported from netssh_rs.netssh_rs")
    except ImportError as e:
        logger.error(f"Error importing modules: {e}")
        logger.error("Make sure you've installed the netssh_rs package: pip install netssh-rs")
        sys.exit(1)

def device_example() -> None:
    """Connect to a device and run commands."""
    
    # Device connection details
    device: Dict[str, Any] = {
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
        net_device = PyNetworkDevice.create(config)
        
        logger.info(f"Connecting to {device['host']}...")
        net_device.connect()
        
        # Run command and get output
        logger.info(f"Running '{command}'...")
        result: str = net_device.send_command(command)
        
        # Print results
        logger.info("Command output:")
        print(result)
        
    except Exception as e:
        logger.error(f"Error during device interaction: {e}")
    
    finally:
        # Clean up the device connection
        if 'net_device' in locals():
            logger.info("Disconnecting...")
            net_device.close()

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