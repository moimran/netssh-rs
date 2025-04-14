#!/usr/bin/env python3
"""
Example script demonstrating how to parse "show version" command output 
from a Cisco IOS device using TextFSM in netssh-rs.

This script connects to a real Cisco device, sends the "show version" command, 
and parses the output using TextFSM to extract structured data.
"""

import sys
import json
import os
import logging

# Configure logging for TextFSM
logging.basicConfig(
    level=logging.DEBUG,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)

# Import both network device functionality and TextFSM parsing from netssh_rs
try:
    from netssh_rs import (
        PyDeviceConfig, 
        PyNetworkDevice, 
        initialize_logging,
        parse_output,
        parse_output_to_json,
        NetworkOutputParser
    )
    # Initialize netssh_rs logging
    initialize_logging()
except ImportError as e:
    print(f"Error importing from netssh_rs: {e}")
    print("Make sure the package is properly installed.")
    sys.exit(1)

def connect_and_parse_show_version():
    """
    Connect to a real Cisco IOS device, run show version, and parse the output.
    
    Returns:
        Parsed output as a dictionary
    """
    # Create device configuration with the provided details
    cisco_device = PyDeviceConfig(
        device_type="cisco_ios",
        host="192.168.1.25", 
        username="admin",
        password="moimran@123",
        port=22,
        timeout_seconds=60,
        secret="moimran@123",
        session_log="/tmp/cisco_session.log"
    )
    
    # Connect to the device
    try:
        command = "show version"  # Changed to show version to match script purpose
        print(f"Connecting to {cisco_device.host}...")
        device = PyNetworkDevice.create(cisco_device)
        device.connect()
        
        # Send the show version command
        print(f"Connected to {cisco_device.host}. Sending {command} command...")
        result = device.send_command(command)
        
        if result.is_success() and result.output is not None:
            print(f"\n----- Raw {command} output -----")
            print(result.output)
            print("-" * 40)
            
            # Parse the output using TextFSM
            print("\nParsing output using TextFSM...")
            parsed_data = parse_output("cisco_ios", command, result.output)
            
            if parsed_data:
                print(f"\nParsed {command} output (JSON format):")
                print(json.dumps(parsed_data, indent=2))
                
                return parsed_data
            else:
                print(f"Error: Failed to parse {command} output")
                return None
        else:
            error_msg = result.error if result.error else "No output received"
            print(f"Error: Command failed - {error_msg}")
            return None
            
    except Exception as e:
        print(f"Error connecting to device: {str(e)}")
        return None
    finally:
        # Close the connection
        if 'device' in locals():
            device.close()
            print("Connection closed.")


def main():
    print("TextFSM Example - Parse Cisco IOS 'show version' Output")
    print("=" * 60)
    
    # Connect to the real device and parse show version output
    connect_and_parse_show_version()


if __name__ == "__main__":
    main() 