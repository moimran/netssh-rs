#!/usr/bin/env python3
"""
Example script for netssh_rs Python bindings.
"""

import os
import sys
import netssh_rs
import netssh_rs.netssh_rs

def main():
    """Main function demonstrating the usage of netssh_rs."""
    # Initialize logging
    netssh_rs.initialize_logging(debug=True, console=True)
    
    # Create a device configuration
    config = netssh_rs.PyDeviceConfig(
        device_type="cisco_ios",
        host="192.168.1.25",
        username="admin",
        password="moimran@123",  # Replace with actual password
        port=22,
        timeout_seconds=60,
        secret="moimran@123",  # Replace with actual enable secret
        session_log="logs/device_session.log"
    )
    
    try:
        # Create and connect to the device
        with netssh_rs.PyNetworkDevice.create(config) as device:
            print("Device created successfully")
            
            try:
                # Connect to the device
                print("Connecting to the device...")
                device.connect()
                print("Connected successfully!")
                
                # Send a command
                print("\nSending command: show version")
                output = device.send_command("show version")
                print(f"Command output:\n{output}")
                
            except Exception as e:
                print(f"Error during device operations: {e}")
    
    except Exception as e:
        print(f"Error creating device: {e}")
        return 1
    
    return 0

if __name__ == "__main__":
    sys.exit(main()) 