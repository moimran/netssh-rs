"""
netssh_parser - A wrapper around netssh-rs with TextFSM parsing

This module provides a high-level interface to the netssh-rs Rust library
with integrated TextFSM parsing.
"""
from typing import Dict, List, Any, Union, Optional, Tuple
import os
import json
import logging
from contextlib import contextmanager

# Import netssh-rs Python module
from netssh_rs import (
    PyDeviceConfig,
    PyNetworkDevice,
    PyParallelExecutionManager,
    initialize_logging
)

# Import TextFSM parser
from textfsm.parse_output import parse_output, parse_output_to_dict

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class NetSSHParser:
    """A wrapper around netssh-rs with TextFSM parsing capabilities."""
    
    def __init__(self, 
                 device_type: str,
                 host: str, 
                 username: str, 
                 password: Optional[str] = None,
                 port: Optional[int] = None,
                 timeout_seconds: Optional[int] = None,
                 secret: Optional[str] = None,
                 session_log: Optional[str] = None,
                 debug: bool = False):
        """
        Initialize a new NetSSHParser instance.
        
        Args:
            device_type: The device type (e.g., cisco_ios, juniper_junos)
            host: The hostname or IP address
            username: SSH username
            password: SSH password (optional if using key-based auth)
            port: SSH port (default: 22)
            timeout_seconds: Connection timeout in seconds
            secret: Enable secret (for Cisco devices)
            session_log: Path to save session logs
            debug: Enable debug logging
        """
        # Initialize logging
        if debug:
            initialize_logging(debug=True, console=True)
        
        # Create device config
        self.config = PyDeviceConfig(
            device_type=device_type,
            host=host,
            username=username,
            password=password,
            port=port,
            timeout_seconds=timeout_seconds,
            secret=secret,
            session_log=session_log
        )
        
        self.device_type = device_type
        self.host = host
        self.device = None
    
    def connect(self) -> None:
        """Connect to the device."""
        if self.device is None:
            self.device = PyNetworkDevice.create(self.config)
        
        self.device.connect()
        self.device.session_preparation()
    
    def close(self) -> None:
        """Close the connection."""
        if self.device is not None:
            self.device.close()
            self.device = None
    
    def send_command(self, command: str, parse: bool = True) -> Union[str, List[Dict[str, str]], None]:
        """
        Send a command to the device and optionally parse the output.
        
        Args:
            command: The command to execute
            parse: Whether to parse the output using TextFSM (default: True)
        
        Returns:
            If parse=True, returns a list of dictionaries with parsed data
            If parse=False, returns the raw command output as a string
        """
        if self.device is None:
            self.connect()
        
        if parse:
            # Use the built-in parsing functionality from netssh-rs
            result = self.device.send_command_with_parse(command, parse=True)
            return result
        else:
            # Just get the raw output
            return self.device.send_command(command)
    
    def send_command_with_metadata(self, command: str) -> Dict[str, Any]:
        """
        Send a command and return both raw and parsed output with metadata.
        
        Args:
            command: The command to execute
        
        Returns:
            A dictionary containing parsed data, raw output, and metadata
        """
        if self.device is None:
            self.connect()
        
        output = self.device.send_command(command)
        return parse_output_to_dict(self.device_type, command, output)
    
    def __enter__(self):
        """Context manager entry."""
        self.connect()
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        self.close()
        return False  # Don't suppress exceptions


class ParallelParser:
    """Run commands on multiple devices in parallel with TextFSM parsing."""
    
    def __init__(self, 
                 max_concurrency: Optional[int] = None,
                 command_timeout_seconds: Optional[int] = None,
                 connection_timeout_seconds: Optional[int] = None,
                 failure_strategy: str = "continue",
                 reuse_connections: bool = True,
                 debug: bool = False):
        """
        Initialize a new ParallelParser instance.
        
        Args:
            max_concurrency: Maximum number of concurrent connections
            command_timeout_seconds: Command execution timeout in seconds
            connection_timeout_seconds: Connection timeout in seconds
            failure_strategy: How to handle failures ("continue", "abort", or "retry")
            reuse_connections: Whether to reuse connections between commands
            debug: Enable debug logging
        """
        # Initialize logging
        if debug:
            initialize_logging(debug=True, console=True)
        
        # Create parallel execution manager
        self.manager = PyParallelExecutionManager(
            max_concurrency=max_concurrency,
            command_timeout_seconds=command_timeout_seconds,
            connection_timeout_seconds=connection_timeout_seconds,
            failure_strategy=failure_strategy,
            reuse_connections=reuse_connections
        )
    
    def execute_command(self, devices: List[Dict[str, Any]], command: str, parse: bool = True) -> Dict[str, Any]:
        """
        Execute a command on multiple devices in parallel.
        
        Args:
            devices: List of device configurations
            command: Command to execute
            parse: Whether to parse the output using TextFSM
        
        Returns:
            Parsed results for all devices
        """
        # Convert device configurations to PyDeviceConfig objects
        device_configs = []
        for device in devices:
            config = PyDeviceConfig(
                device_type=device['device_type'],
                host=device['host'],
                username=device['username'],
                password=device.get('password'),
                port=device.get('port'),
                timeout_seconds=device.get('timeout_seconds'),
                secret=device.get('secret'),
                session_log=device.get('session_log')
            )
            device_configs.append(config)
        
        # Execute command on all devices with parsing
        results = self.manager.execute_command_on_all_with_parse(
            device_configs, command, parse=parse
        )
        
        if parse:
            # Parse all outputs if not already parsed
            results.parse_all_outputs()
            
            # Get dictionary of parsed results
            return {
                'parsed': results.get_parsed_outputs(),
                'raw': results.get_all_results()
            }
        else:
            # Just return raw results
            return {
                'raw': results.get_all_results()
            }
    
    def execute_commands(self, 
                        device_commands: Dict[str, List[str]], 
                        parse: bool = True) -> Dict[str, Any]:
        """
        Execute multiple commands on multiple devices.
        
        Args:
            device_commands: Dictionary mapping device configs to lists of commands
            parse: Whether to parse the output
        
        Returns:
            Parsed results for all devices and commands
        """
        # Need to implement the conversion from Python dict to PyDict
        # This would depend on the implementation details of execute_commands_with_parse
        raise NotImplementedError("This method is not yet implemented")
    
    def __enter__(self):
        """Context manager entry."""
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        self.manager.cleanup()
        return False  # Don't suppress exceptions


@contextmanager
def connect(device_type: str,
           host: str,
           username: str,
           password: Optional[str] = None,
           port: Optional[int] = None,
           timeout_seconds: Optional[int] = None,
           secret: Optional[str] = None,
           session_log: Optional[str] = None,
           debug: bool = False):
    """
    A context manager for connecting to a device.
    
    Args:
        device_type: The device type (e.g., cisco_ios, juniper_junos)
        host: The hostname or IP address
        username: SSH username
        password: SSH password (optional if using key-based auth)
        port: SSH port (default: 22)
        timeout_seconds: Connection timeout in seconds
        secret: Enable secret (for Cisco devices)
        session_log: Path to save session logs
        debug: Enable debug logging
    
    Yields:
        A connected NetSSHParser instance
    """
    parser = NetSSHParser(
        device_type=device_type,
        host=host,
        username=username,
        password=password,
        port=port,
        timeout_seconds=timeout_seconds,
        secret=secret,
        session_log=session_log,
        debug=debug
    )
    
    try:
        parser.connect()
        yield parser
    finally:
        parser.close()


def send_command(device_type: str,
                host: str,
                username: str,
                command: str,
                password: Optional[str] = None,
                port: Optional[int] = None,
                timeout_seconds: Optional[int] = None,
                secret: Optional[str] = None,
                session_log: Optional[str] = None,
                parse: bool = True,
                debug: bool = False) -> Union[str, List[Dict[str, str]], None]:
    """
    Connect to a device, send a command, and optionally parse the output.
    
    Args:
        device_type: The device type (e.g., cisco_ios, juniper_junos)
        host: The hostname or IP address
        username: SSH username
        command: The command to execute
        password: SSH password (optional if using key-based auth)
        port: SSH port (default: 22)
        timeout_seconds: Connection timeout in seconds
        secret: Enable secret (for Cisco devices)
        session_log: Path to save session logs
        parse: Whether to parse the output using TextFSM
        debug: Enable debug logging
    
    Returns:
        If parse=True, returns a list of dictionaries with parsed data
        If parse=False, returns the raw command output as a string
    """
    with connect(
        device_type=device_type,
        host=host,
        username=username,
        password=password,
        port=port,
        timeout_seconds=timeout_seconds,
        secret=secret,
        session_log=session_log,
        debug=debug
    ) as device:
        return device.send_command(command, parse=parse)


def main():
    """Example usage of the module."""
    import argparse
    
    parser = argparse.ArgumentParser(description="Send commands to network devices with TextFSM parsing")
    parser.add_argument("--device_type", required=True, help="Device type (e.g., cisco_ios)")
    parser.add_argument("--host", required=True, help="Hostname or IP address")
    parser.add_argument("--username", required=True, help="SSH username")
    parser.add_argument("--password", help="SSH password")
    parser.add_argument("--port", type=int, help="SSH port")
    parser.add_argument("--command", required=True, help="Command to execute")
    parser.add_argument("--no-parse", action="store_true", help="Don't parse the output")
    parser.add_argument("--debug", action="store_true", help="Enable debug logging")
    
    args = parser.parse_args()
    
    result = send_command(
        device_type=args.device_type,
        host=args.host,
        username=args.username,
        password=args.password,
        port=args.port,
        command=args.command,
        parse=not args.no_parse,
        debug=args.debug
    )
    
    if isinstance(result, list):
        print(json.dumps(result, indent=2))
    else:
        print(result)


if __name__ == "__main__":
    main() 