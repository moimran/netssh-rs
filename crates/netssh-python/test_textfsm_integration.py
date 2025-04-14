#!/usr/bin/env python3
"""
Test script to validate TextFSM integration with netssh-rs.

This script checks the following:
1. TextFSM module is accessible through netssh_rs
2. The template directory structure is correct
3. The parser can successfully parse sample output
"""

import os
import sys
import json

try:
    # Verify we can import TextFSM utilities from netssh_rs
    from netssh_rs import parse_output, parse_output_to_json, NetworkOutputParser
    print("✓ Successfully imported TextFSM parser from netssh_rs")
except ImportError as e:
    print(f"✗ Failed to import TextFSM parser from netssh_rs: {e}")
    sys.exit(1)

# Sample Cisco IOS "show version" output for testing
SAMPLE_OUTPUT = """
Cisco IOS Software, C2960 Software (C2960-LANBASEK9-M), Version 15.0(2)SE11, RELEASE SOFTWARE (fc3)
Technical Support: http://www.cisco.com/techsupport
Copyright (c) 1986-2017 by Cisco Systems, Inc.
Compiled Thu 30-Mar-17 01:44 by prod_rel_team

ROM: Bootstrap program is C2960 boot loader
BOOTLDR: C2960 Boot Loader (C2960-HBOOT-M) Version 12.2(53r)SEY4, RELEASE SOFTWARE (fc1)

Switch uptime is 3 years, 24 weeks, 6 days, 8 hours, 42 minutes
System returned to ROM by power-on
System restarted at 10:52:13 UTC Mon Mar 1 2021
System image file is "flash:/c2960-lanbasek9-mz.150-2.SE11.bin"
Last reload reason: power-on


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

cisco WS-C2960-24TT-L (PowerPC405) processor (revision D0) with 65536K bytes of memory.
Processor board ID FOC1350W1JH
Last reset from power-on
1 Virtual Ethernet interface
24 FastEthernet interfaces
2 Gigabit Ethernet interfaces
The password-recovery mechanism is enabled.

64K bytes of flash-simulated non-volatile configuration memory.
Base ethernet MAC Address       : 00:1C:B0:A1:B2:C3
Motherboard assembly number     : 73-10390-04
Power supply part number        : 341-0097-02
Motherboard serial number       : FOC13494MWL
Power supply serial number      : AZS1349018M
Model revision number           : D0
Motherboard revision number     : A0
Model number                    : WS-C2960-24TT-L
System serial number            : FOC1350W1JH
Top Assembly Part Number        : 800-27221-02
Top Assembly Revision Number    : A0
Version ID                      : V04
CLEI Code Number                : COM1X00ARB
Hardware Board Revision Number  : 0x0D


Switch Ports Model                     SW Version            SW Image                 
------ ----- -----                     ----------            ----------               
*    1 26    WS-C2960-24TT-L           15.0(2)SE11           C2960-LANBASEK9-M        


Configuration register is 0xF
"""

def test_parser():
    """
    Test the TextFSM parser functionality.
    """
    # Create a parser instance with default template directory
    parser = NetworkOutputParser()
    
    # Get the template directory path
    template_dir = parser.template_dir
    print(f"Template directory: {template_dir}")
    
    # Check if the template directory exists
    if os.path.isdir(template_dir):
        print(f"✓ Template directory exists")
    else:
        print(f"✗ Template directory not found: {template_dir}")
        return False
    
    # Check that the index file exists
    index_file = os.path.join(template_dir, "index")
    if os.path.isfile(index_file):
        print(f"✓ Template index file exists")
    else:
        print(f"✗ Template index file not found: {index_file}")
        return False
    
    # Check for Cisco IOS show version template
    cisco_ios_template = "cisco_ios_show_version.textfsm"
    template_path = os.path.join(template_dir, cisco_ios_template)
    
    # Try to find the template using the parser
    found_template = parser.find_template("cisco_ios", "show version")
    if found_template:
        print(f"✓ Found template for 'cisco_ios show version': {found_template}")
    else:
        print(f"✗ Failed to find template for 'cisco_ios show version'")
        return False
    
    # Test parsing sample output
    try:
        parsed_data = parser.parse_output("cisco_ios", "show version", SAMPLE_OUTPUT)
        if parsed_data:
            print(f"✓ Successfully parsed sample output")
            print("\nParsed data (first item):")
            print(json.dumps(parsed_data[0], indent=2))
            
            # Verify expected fields are present
            expected_fields = ["VERSION", "HOSTNAME", "UPTIME", "HARDWARE", "SERIAL"]
            missing_fields = [field for field in expected_fields if field not in parsed_data[0]]
            
            if not missing_fields:
                print(f"✓ All expected fields found in parsed data")
            else:
                print(f"✗ Missing expected fields: {', '.join(missing_fields)}")
                return False
                
            return True
        else:
            print(f"✗ Parser returned None (parsing failed)")
            return False
    except Exception as e:
        print(f"✗ Exception while parsing: {str(e)}")
        return False

def main():
    """
    Main test function
    """
    print("Testing TextFSM integration with netssh-rs")
    print("=" * 50)
    
    result = test_parser()
    
    if result:
        print("\n✓ TextFSM integration test PASSED")
        sys.exit(0)
    else:
        print("\n✗ TextFSM integration test FAILED")
        sys.exit(1)

if __name__ == "__main__":
    main() 