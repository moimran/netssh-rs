#!/usr/bin/env python3
"""
Example demonstrating TextFSM parsing capabilities for network command outputs.

This example shows how to:
1. Parse command outputs using TextFSM templates
2. Work with structured data from parsed outputs
3. Get information about available templates
"""

import os
import sys
from pprint import pprint

# Add the parent directory to the Python path to find the modules
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# Import the textfsm parsing module
from textfsm.parse import parse_output, ParsingException

def get_available_templates():
    """Get a list of available TextFSM templates."""
    template_dir = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), 
                              "textfsm", "templates")
    
    templates = []
    if os.path.exists(template_dir):
        for filename in os.listdir(template_dir):
            if filename.endswith(".textfsm"):
                templates.append(filename)
    
    return templates

def get_platforms():
    """Get a list of supported platforms based on available templates."""
    template_dir = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), 
                              "textfsm", "templates")
    
    index_file = os.path.join(template_dir, "index")
    platforms = set()
    
    if os.path.exists(index_file):
        with open(index_file, 'r') as f:
            for line in f:
                if line.startswith('#') or not line.strip():
                    continue
                    
                parts = line.strip().split(',')
                if len(parts) >= 3:
                    platform = parts[2].strip()
                    platforms.add(platform)
    
    return sorted(list(platforms))

def get_supported_commands(platform):
    """Get a list of commands supported by TextFSM templates for a specific platform."""
    template_dir = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), 
                              "textfsm", "templates")
    
    index_file = os.path.join(template_dir, "index")
    commands = []
    
    if os.path.exists(index_file):
        with open(index_file, 'r') as f:
            for line in f:
                if line.startswith('#') or not line.strip():
                    continue
                    
                parts = line.strip().split(',')
                if len(parts) >= 4:
                    template_platform = parts[2].strip()
                    if template_platform == platform:
                        command = parts[3].strip()
                        commands.append(command)
    
    return commands

def main():
    # Sample command output
    cisco_ios_show_version = """
Cisco IOS Software, C3560 Software (C3560-IPSERVICESK9-M), Version 12.2(55)SE7, RELEASE SOFTWARE (fc1)
Technical Support: http://www.cisco.com/techsupport
Copyright (c) 1986-2013 by Cisco Systems, Inc.
Compiled Mon 28-Jan-13 10:10 by prod_rel_team

ROM: Bootstrap program is C3560 boot loader
BOOTLDR: C3560 Boot Loader (C3560-HBOOT-M) Version 12.2(44)SE5, RELEASE SOFTWARE (fc1)

Switch uptime is 1 year, 2 weeks, 4 days, 6 hours, 32 minutes
System returned to ROM by power-on
System restarted at 04:05:29 cdt Thu Mar 24 2023
System image file is "flash:c3560-ipservicesk9-mz.122-55.SE7.bin"
Last reload type: Normal Reload


This product contains cryptographic features and is subject to United
States and local country laws governing import, export, transfer and
use. Delivery of Cisco cryptographic products does not imply
third-party authority to import, export, distribute or use encryption.
Importers, exporters, distributors and users are responsible for
compliance with U.S. and local country laws. By using this product you
agree to comply with applicable laws and regulations. If you are unable
to comply with U.S. and local laws, return this product immediately.

A summary of U.S. laws governing Cisco cryptographic products may be found at:
http://www.cisco.com/wwl/export/crypto/tool/stqrg.html

If you require further assistance please contact us by sending email to
export@cisco.com.

cisco WS-C3560-24PS (PowerPC405) processor (revision F0) with 131072K bytes of memory.
Processor board ID CAT0933NLQQ
Last reset from power-on
3 Virtual Ethernet interfaces
24 FastEthernet interfaces
2 Gigabit Ethernet interfaces
The password-recovery mechanism is enabled.

512K bytes of flash-simulated non-volatile configuration memory.
Base ethernet MAC Address       : 00:23:EB:A3:96:80
Motherboard assembly number     : 73-10390-09
Power supply part number        : 341-0097-03
Motherboard serial number       : CAT09290N33
Power supply serial number      : DTH0921259M
Model revision number           : F0
Motherboard revision number     : A0
Model number                    : WS-C3560-24PS-S
System serial number            : CAT0933NLQQ
Top Assembly Part Number        : 800-27561-04
Top Assembly Revision Number    : A0
Version ID                      : V03
CLEI Code Number                : COMM1S00BRA
Hardware Board Revision Number  : 0x09


Switch Ports Model              SW Version            SW Image
------ ----- -----              ----------            ----------
*    1 26    WS-C3560-24PS      12.2(55)SE7           C3560-IPSERVICESK9-M

Configuration register is 0xF

    """
    
    arista_eos_show_version = """
Arista DCS-7280SR-48C6-R
Hardware version: 11.01
Serial number: JPE17194058
Hardware MAC address: 2899.3a06.b4e1
System MAC address: 2899.3a06.b4e1

Software image version: 4.21.3F
Architecture: i686
Internal build version: 4.21.3F-10213456.4213F
Internal build ID: 35964f61-92bd-4cfd-b7da-c74d0b2ed5d8

Uptime: 12 weeks, 3 days, 23 hours and 48 minutes
Total memory: 8051592 kB
Free memory: 5254208 kB
    """
    
    print("=== TextFSM Template Information ===")
    
    # Get available templates
    templates = get_available_templates()
    print(f"Available templates: {len(templates)}")
    if templates:
        print("First 5 templates:")
        for template in templates[:5]:
            print(f"  - {template}")
    
    # Get supported platforms
    platforms = get_platforms()
    print(f"\nSupported platforms: {len(platforms)}")
    print(f"Platforms: {', '.join(platforms[:10])}...")
    
    # Get commands for a specific platform
    platform = "cisco_ios"
    commands = get_supported_commands(platform)
    print(f"\nCommands supported for {platform}: {len(commands)}")
    if commands:
        print("First 5 commands:")
        for command in commands[:5]:
            print(f"  - {command}")
    
    print("\n=== Parsing Cisco IOS 'show version' output ===")
    try:
        # Parse the sample output
        parsed_data = parse_output(
            platform="cisco_ios",
            command="show version",
            data=cisco_ios_show_version
        )
        
        # Print the structured data
        print("\nParsed data (structured):")
        pprint(parsed_data)
        
        # Working with structured data
        if parsed_data:
            entry = parsed_data[0]  # First entry
            print("\nExtracted information:")
            print(f"  Version: {entry.get('version', 'N/A')}")
            print(f"  Hardware: {entry.get('hardware', 'N/A')}")
            print(f"  Uptime: {entry.get('uptime', 'N/A')}")
            print(f"  Serial Number: {entry.get('serial', 'N/A')}")
    except ParsingException as e:
        print(f"Error parsing Cisco IOS output: {e}")
    
    print("\n=== Parsing Arista EOS 'show version' output ===")
    try:
        # Parse the sample output
        parsed_data = parse_output(
            platform="arista_eos",
            command="show version",
            data=arista_eos_show_version
        )
        
        # Print the structured data
        print("\nParsed data (structured):")
        pprint(parsed_data)
        
        # Working with structured data
        if parsed_data:
            entry = parsed_data[0]  # First entry
            print("\nExtracted information:")
            print(f"  Model: {entry.get('model', 'N/A')}")
            print(f"  Version: {entry.get('version', 'N/A')}")
            print(f"  Serial Number: {entry.get('serial_number', 'N/A')}")
            print(f"  Uptime: {entry.get('uptime', 'N/A')}")
    except ParsingException as e:
        print(f"Error parsing Arista EOS output: {e}")

if __name__ == "__main__":
    main() 