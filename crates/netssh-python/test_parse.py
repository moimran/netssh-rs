#!/usr/bin/env python3
"""
Test script for netssh-rs TextFSM parsing functionality.
This script demonstrates how to use the TextFSM parsing with netssh-rs.
"""
import os
import sys
import argparse
import json
import logging
from typing import Dict, Any, List, Optional, Union, cast

# Configure logging
logging.basicConfig(level=logging.INFO, 
                    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s')
logger = logging.getLogger("test_parser")

# Add the current directory to the path to ensure modules can be found
sys.path.append(os.path.dirname(os.path.abspath(__file__)))

# Import flags
NETPARSER_AVAILABLE = False
NETSSH_RS_AVAILABLE = False
TEXTFSM_PARSER_AVAILABLE = False

# Try importing the available modules
try:
    # Check for textfsm parsing module
    from textfsm.parse_output import parse_output, parse_output_to_dict
    TEXTFSM_PARSER_AVAILABLE = True
    logger.info("TextFSM parse_output module loaded successfully")
except ImportError as e:
    logger.warning(f"TextFSM parse_output module not available: {e}")

try:
    # Try importing from the netssh_parser module
    from netssh_parser import NetSSHParser, send_command, connect
    logger.info("netssh_parser module loaded successfully")
    NETPARSER_AVAILABLE = True
except ImportError as e:
    logger.warning(f"netssh_parser module not available: {e}")

try:
    # Try direct imports from netssh_rs
    import netssh_rs
    logger.info("netssh_rs module loaded successfully")
    NETSSH_RS_AVAILABLE = True
except ImportError as e:
    logger.warning(f"netssh_rs module not available: {e}")

if not (TEXTFSM_PARSER_AVAILABLE or NETPARSER_AVAILABLE or NETSSH_RS_AVAILABLE):
    logger.error("No required modules available. Make sure netssh_rs and TextFSM are properly installed.")
    sys.exit(1)


def test_with_sample_output(device_type: str, command: str, output_file: str) -> None:
    """Test parsing with a sample output file."""
    logger.info(f"Testing parsing for {device_type} command: {command}")
    
    # Read sample output
    try:
        with open(output_file, 'r') as f:
            output = f.read()
        
        logger.info(f"Read {len(output)} bytes from {output_file}")
    except Exception as e:
        logger.error(f"Error reading file {output_file}: {e}")
        return
    
    # Parse using TextFSM
    if not TEXTFSM_PARSER_AVAILABLE:
        logger.error("TextFSM parse_output module not available")
        return
        
    try:
        parsed_data = parse_output(device_type, command, output)
        
        if parsed_data:
            logger.info(f"Successfully parsed output into {len(parsed_data)} records")
            print("\nSample parsed data (first record):")
            print(json.dumps(parsed_data[0], indent=2))
            
            # Get full data with metadata
            full_data = parse_output_to_dict(device_type, command, output)
            print(f"\nFull metadata includes:")
            print(f"- Device type: {full_data.get('device_type', 'N/A')}")
            print(f"- Command: {full_data.get('command', 'N/A')}")
            print(f"- Template: {full_data.get('template', 'N/A')}")
            print(f"- Status: {full_data.get('status', 'N/A')}")
            
            if full_data.get('header'):
                print(f"- Headers: {', '.join(full_data['header'])}")
        else:
            logger.warning("No data parsed. Check if a template exists for this command.")
    except Exception as e:
        logger.error(f"Error parsing: {e}")


def print_output_sample(output: Union[str, List, None]) -> None:
    """Print a sample of the output with appropriate handling of different types."""
    if output is None:
        logger.warning("Output is None")
        return
        
    if isinstance(output, str):
        # Handle string output
        print("\nRaw Output Sample:")
        print("-" * 40)
        if len(output) > 200:
            print(f"{output[:200]}...")
        else:
            print(output)
        print("-" * 40)
    elif isinstance(output, list):
        # Handle list output
        print("\nStructured Output Sample:")
        print("-" * 40)
        print(json.dumps(output[:1], indent=2))
        print("-" * 40)
    else:
        # Handle any other type
        print(f"\nOutput (type: {type(output)}):")
        print("-" * 40)
        try:
            print(json.dumps(output, indent=2))
        except (TypeError, ValueError):
            print(str(output))
        print("-" * 40)


def test_with_live_device(
        device_type: str, 
        host: str, 
        username: str, 
        password: Optional[str],
        command: str,
        port: Optional[int] = None
    ) -> None:
    """Test parsing with a live device connection."""
    logger.info(f"Testing live connection to {host}")
    
    # First try using the high-level netssh_parser if available
    if NETPARSER_AVAILABLE:
        logger.info("Using NetSSHParser for connection")
        try:
            with connect(
                device_type=device_type,
                host=host,
                username=username,
                password=password,
                port=port,
                debug=True
            ) as device:
                logger.info(f"Connected successfully. Sending command: {command}")
                
                # Get raw output
                raw_output = device.send_command(command)
                if raw_output:
                    logger.info(f"Received output of type {type(raw_output)}")
                    print_output_sample(raw_output)
                else:
                    logger.warning("Received empty output")
                
                # Parse output
                if TEXTFSM_PARSER_AVAILABLE:
                    logger.info("Parsing output with TextFSM")
                    # Ensure raw_output is a string
                    if isinstance(raw_output, str):
                        parsed_data = parse_output(device_type, command, raw_output)
                        
                        if parsed_data:
                            logger.info(f"Successfully parsed into {len(parsed_data)} records")
                            print("\nParsed Data Sample (first record):")
                            print("-" * 40)
                            print(json.dumps(parsed_data[0], indent=2))
                            print("-" * 40)
                        else:
                            logger.warning("No data parsed. Check if a template exists for this command.")
                    else:
                        logger.warning("Cannot parse non-string output with TextFSM")
                else:
                    logger.warning("TextFSM parsing not available")
                
            return  # Successful connection and execution
        except Exception as e:
            logger.error(f"Error using NetSSHParser: {e}")
            # Fall through to try the next method
    
    # If netssh_parser is not available or failed, try direct netssh_rs if available
    if NETSSH_RS_AVAILABLE:
        try:
            # We don't use the linter-reported symbols directly to avoid errors
            # Instead, import dynamically from the module we already checked
            PyDeviceConfig = getattr(netssh_rs, 'PyDeviceConfig')
            PyNetworkDevice = getattr(netssh_rs, 'PyNetworkDevice')
            initialize_logging = getattr(netssh_rs, 'initialize_logging')
            
            logger.info("Using PyNetworkDevice directly")
            
            # Initialize logging
            initialize_logging(debug=True, console=True)
            
            # Create device config
            config = PyDeviceConfig(
                device_type=device_type,
                host=host,
                username=username,
                password=password,
                port=port
            )
            
            # Create and connect to device
            device = PyNetworkDevice.create(config)
            device.connect()
            device.session_preparation()
            
            try:
                logger.info(f"Connected successfully. Sending command: {command}")
                
                # Get raw output
                raw_output = device.send_command(command)
                if raw_output:
                    logger.info(f"Received output of type {type(raw_output)}")
                    print_output_sample(raw_output)
                else:
                    logger.warning("Received empty output")
                
                # Parse output with TextFSM if available
                if TEXTFSM_PARSER_AVAILABLE:
                    logger.info("Parsing output with TextFSM")
                    # Ensure raw_output is a string
                    if isinstance(raw_output, str):
                        parsed_data = parse_output(device_type, command, raw_output)
                        
                        if parsed_data:
                            logger.info(f"Successfully parsed into {len(parsed_data)} records")
                            print("\nParsed Data Sample (first record):")
                            print("-" * 40)
                            print(json.dumps(parsed_data[0], indent=2))
                            print("-" * 40)
                        else:
                            logger.warning("No data parsed. Check if a template exists for this command.")
                    else:
                        logger.warning("Cannot parse non-string output with TextFSM")
                else:
                    logger.warning("TextFSM parsing not available")
                    
            finally:
                device.close()
                logger.info("Connection closed")
                
            return  # Successful connection and execution
            
        except Exception as e:
            logger.error(f"Error using PyNetworkDevice directly: {e}")
    
    # If we reach here, neither method worked
    logger.error("Unable to connect to the device. Make sure netssh_rs or netssh_parser is properly installed.")


def main():
    parser = argparse.ArgumentParser(description="Test netssh-rs TextFSM parsing")
    
    # Create subparsers for different test modes
    subparsers = parser.add_subparsers(dest='mode', help='Test mode')
    
    # Sample output test
    sample_parser = subparsers.add_parser('sample', help='Test with sample output file')
    sample_parser.add_argument('--device-type', required=True, help='Device type (e.g., cisco_ios)')
    sample_parser.add_argument('--command', required=True, help='Command to parse')
    sample_parser.add_argument('--output-file', required=True, help='File containing sample output')
    
    # Live device test
    live_parser = subparsers.add_parser('live', help='Test with live device')
    live_parser.add_argument('--device-type', required=True, help='Device type (e.g., cisco_ios)')
    live_parser.add_argument('--host', required=True, help='Host to connect to')
    live_parser.add_argument('--username', required=True, help='Username')
    live_parser.add_argument('--password', help='Password')
    live_parser.add_argument('--port', type=int, help='SSH port')
    live_parser.add_argument('--command', required=True, help='Command to execute')
    
    args = parser.parse_args()
    
    if not args.mode:
        parser.print_help()
        return
    
    if args.mode == 'sample':
        test_with_sample_output(args.device_type, args.command, args.output_file)
    elif args.mode == 'live':
        test_with_live_device(
            args.device_type, 
            args.host, 
            args.username, 
            args.password,
            args.command,
            args.port
        )


if __name__ == "__main__":
    main() 