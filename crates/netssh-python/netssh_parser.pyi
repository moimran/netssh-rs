"""
Type stubs for netssh_parser module.

This file provides type hints for the netssh_parser module to enable proper
IntelliSense support in Python editors.
"""

from typing import Dict, List, Any, Union, Optional, Tuple, Iterator, ContextManager
from typing_extensions import Literal
from types import TracebackType

# These are stub-only imports to prevent linter errors
# The actual implementation imports these from netssh_rs
class PyDeviceConfig:
    device_type: str
    host: str
    username: str
    password: Optional[str]
    port: Optional[int]
    timeout_seconds: Optional[int]
    secret: Optional[str]
    session_log: Optional[str]

class PyNetworkDevice:
    @classmethod
    def create(cls, config: PyDeviceConfig) -> "PyNetworkDevice": ...
    def connect(self) -> None: ...
    def close(self) -> None: ...
    def send_command(self, command: str) -> str: ...

class PyCommandResult:
    device_id: str
    device_type: str
    command: str
    output: str
    start_time: str
    end_time: str
    duration_ms: int
    status: str
    error: Optional[str]

class PyBatchCommandResults:
    def get_device_results(self, device_id: str) -> Optional[List[PyCommandResult]]: ...
    def get_all_results(self) -> List[PyCommandResult]: ...
    def get_successful_results(self) -> List[PyCommandResult]: ...
    def get_failed_results(self) -> List[PyCommandResult]: ...
    def get_command_results(self, command: str) -> List[PyCommandResult]: ...

class PyParallelExecutionManager:
    def __init__(
        self,
        max_concurrency: Optional[int] = None,
        command_timeout_seconds: Optional[int] = None,
        connection_timeout_seconds: Optional[int] = None,
        failure_strategy: Optional[Literal["continue", "abort", "skip_device", "skip_command"]] = None,
        reuse_connections: Optional[bool] = None
    ) -> None: ...
    def execute_command_on_all(self, devices: List[PyDeviceConfig], command: str) -> PyBatchCommandResults: ...

class NetSSHParser:
    """A wrapper around netssh-rs with TextFSM parsing capabilities."""
    
    config: PyDeviceConfig
    device_type: str
    host: str
    device: Optional[PyNetworkDevice]
    
    def __init__(
        self,
        device_type: str,
        host: str,
        username: str,
        password: Optional[str] = None,
        port: Optional[int] = None,
        timeout_seconds: Optional[int] = None,
        secret: Optional[str] = None,
        session_log: Optional[str] = None,
        debug: bool = False
    ) -> None:
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
        ...
    
    def connect(self) -> None:
        """Connect to the device."""
        ...
    
    def close(self) -> None:
        """Close the connection."""
        ...
    
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
        ...
    
    def send_command_textfsm(self, command: str) -> List[Dict[str, str]]:
        """
        Send a command to the device and parse the output with TextFSM.
        
        Args:
            command: The command to execute
        
        Returns:
            A list of dictionaries containing the parsed output
            Each dictionary represents a row of data with field names as keys
        """
        ...
    
    def parse_output(self, command: str, output: str) -> List[Dict[str, str]]:
        """
        Parse command output using TextFSM.
        
        Args:
            command: The command that was executed
            output: The command output to parse
        
        Returns:
            A list of dictionaries containing the parsed output
        """
        ...
    
    def __enter__(self) -> "NetSSHParser":
        """Context manager entry."""
        ...
    
    def __exit__(
        self,
        exc_type: Optional[type],
        exc_value: Optional[Exception],
        exc_tb: Optional[TracebackType]
    ) -> bool:
        """Context manager exit."""
        ...

class ParallelParsingManager(PyParallelExecutionManager):
    """A wrapper around PyParallelExecutionManager with TextFSM parsing capabilities."""
    
    def __init__(
        self,
        max_concurrency: Optional[int] = None,
        command_timeout_seconds: Optional[int] = None,
        connection_timeout_seconds: Optional[int] = None,
        failure_strategy: Optional[Literal["continue", "abort", "skip_device", "skip_command"]] = None,
        reuse_connections: Optional[bool] = None,
        parse_results: bool = True
    ) -> None:
        """
        Initialize a parallel parsing manager.
        
        Args:
            max_concurrency: Maximum number of concurrent connections
            command_timeout_seconds: Command timeout in seconds
            connection_timeout_seconds: Connection timeout in seconds
            failure_strategy: Strategy for handling failures
            reuse_connections: Whether to reuse connections for multiple commands
            parse_results: Whether to automatically parse results with TextFSM
        """
        ...
    
    def execute_command_on_all(
        self, 
        devices: List[PyDeviceConfig], 
        command: str,
        parse: bool = True
    ) -> PyBatchCommandResults:
        """
        Execute a command on all devices with optional parsing.
        
        Args:
            devices: List of device configurations
            command: Command to execute
            parse: Whether to parse the output using TextFSM
            
        Returns:
            Batch command results with parsed outputs if parse=True
        """
        ...
    
    def execute_commands_on_all(
        self, 
        devices: List[PyDeviceConfig], 
        commands: List[str],
        parse: bool = True
    ) -> PyBatchCommandResults:
        """
        Execute multiple commands on all devices with optional parsing.
        
        Args:
            devices: List of device configurations
            commands: List of commands to execute
            parse: Whether to parse the output using TextFSM
            
        Returns:
            Batch command results with parsed outputs if parse=True
        """
        ...
    
    def execute_commands(
        self, 
        device_commands: Dict[PyDeviceConfig, Union[str, List[str]]],
        parse: bool = True
    ) -> PyBatchCommandResults:
        """
        Execute specific commands on specific devices with optional parsing.
        
        Args:
            device_commands: Dictionary mapping device configurations to commands or lists of commands
            parse: Whether to parse the output using TextFSM
            
        Returns:
            Batch command results with parsed outputs if parse=True
        """
        ...

def connect(
    device_type: str,
    host: str,
    username: str,
    password: Optional[str] = None,
    port: Optional[int] = None,
    timeout_seconds: Optional[int] = None,
    secret: Optional[str] = None,
    session_log: Optional[str] = None,
    debug: bool = False
) -> ContextManager[NetSSHParser]:
    """
    Context manager for connecting to a device.
    
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
    
    Returns:
        A context manager that yields a connected NetSSHParser instance
    """
    ...

def send_command(
    device_type: str,
    host: str,
    username: str,
    command: str,
    password: Optional[str] = None,
    port: Optional[int] = None,
    secret: Optional[str] = None,
    parse: bool = True
) -> Union[str, List[Dict[str, str]]]:
    """
    Send a command to a device and optionally parse the output.
    
    This is a convenience function that handles connection setup and teardown.
    
    Args:
        device_type: The device type (e.g., cisco_ios, juniper_junos)
        host: The hostname or IP address
        username: SSH username
        command: The command to execute
        password: SSH password (optional if using key-based auth)
        port: SSH port (default: 22)
        secret: Enable secret (for Cisco devices)
        parse: Whether to parse the output using TextFSM (default: True)
    
    Returns:
        If parse=True, returns a list of dictionaries with parsed data
        If parse=False, returns the raw command output as a string
    """
    ... 