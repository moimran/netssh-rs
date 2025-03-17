#!/usr/bin/env python3
"""
Example of using netssh_rs from Python.

This example demonstrates how to connect to a Cisco IOS device,
send commands, and configure an interface.
"""

import netssh_rs

def main():
    # Initialize logging
    netssh_rs.initialize_logging(debug=False, console=True)
    
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
    
    # Create a device using the factory
    print("Creating device...")
    device = netssh_rs.PyNetworkDevice.create(config)
    
    # Use context manager to ensure connection is closed
    with device:
        # Connect to the device
        print("Connecting to device...")
        device.connect()
        
        # Get device output
        print("Sending commands...")
        show_run = device.send_command("show run")
        print(show_run)
        
        # # Configure an interface
        # print("Configuring interface...")
        # device.enter_config_mode(None)
        # device.send_command("interface GigabitEthernet1")
        # device.send_command("description Configured by Python using netssh_rs")
        # device.exit_config_mode(None)
        
        # # Save configuration
        # device.save_configuration()
        
        # # Get more output
        # show_run_after = device.send_command("show run")
        # print(show_run_after)
        
        # show_ver = device.send_command("show version")
        # print(show_ver)

        device.close()
    
    print("Done!")

if __name__ == "__main__":
    main()