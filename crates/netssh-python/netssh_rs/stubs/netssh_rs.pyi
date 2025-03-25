"""
Type stubs for netssh_rs module.

This file provides type hints for the netssh_rs module to enable proper
IntelliSense support in Python editors.
"""

from typing import Dict, List, Any, Optional, Union, Type, TypeVar
from types import TracebackType

T = TypeVar('T', bound='PyNetworkDevice')

def initialize_logging(debug: bool = False, console: bool = False) -> None:
    """Initialize logging for the netssh_rs module."""
    ...

class PyDeviceConfig:
    """Configuration for a network device connection."""
    
    device_type: str
    host: str
    username: str
    password: Optional[str]
    port: Optional[int]
    timeout_seconds: Optional[int]
    secret: Optional[str]
    session_log: Optional[str]
    
    def __init__(
        self, 
        device_type: str,
        host: str,
        username: str,
        password: Optional[str] = None,
        port: Optional[int] = None,
        timeout_seconds: Optional[int] = None,
        secret: Optional[str] = None,
        session_log: Optional[str] = None
    ) -> None:
        """
        Initialize a new device configuration.
        
        Args:
            device_type: The type of device (e.g., 'cisco_ios', 'juniper')
            host: The hostname or IP address of the device
            username: The username for authentication
            password: The password for authentication
            port: The SSH port (default: 22)
            timeout_seconds: Connection timeout in seconds
            secret: The enable secret for privileged mode (if required)
            session_log: Path to log file for session logging
        """
        ...
    
    @property
    def device_type(self) -> str: ...
    
    @property
    def host(self) -> str: ...
    
    @property
    def username(self) -> str: ...
    
    @property
    def password(self) -> str: ...
    
    @property
    def port(self) -> int: ...
    
    @property
    def secret(self) -> Optional[str]: ...
    
    @property
    def session_log(self) -> Optional[str]: ...

class PyDeviceInfo:
    """Information about a network device."""
    
    vendor: str
    model: str
    os_version: str
    hostname: str
    uptime: str

class PyNetworkDevice:
    """Network device connection handler."""
    
    @classmethod
    def create(cls: Type[T], config: PyDeviceConfig) -> T:
        """
        Create a new network device connection.
        
        Args:
            config: The device configuration
            
        Returns:
            A new PyNetworkDevice instance
        """
        ...
    
    def connect(self) -> None:
        """
        Connect to the device.
        
        Raises:
            ConnectionError: If connection fails
        """
        ...
    
    def close(self) -> None:
        """Close the connection to the device."""
        ...
    
    def check_config_mode(self) -> bool:
        """
        Check if the device is in configuration mode.
        
        Returns:
            True if in config mode, False otherwise
        """
        ...
    
    def enter_config_mode(self, config_command: Optional[str] = None) -> None:
        """
        Enter configuration mode.
        
        Args:
            config_command: Custom config command if needed
        """
        ...
    
    def exit_config_mode(self, exit_command: Optional[str] = None) -> None:
        """
        Exit configuration mode.
        
        Args:
            exit_command: Custom exit command if needed
        """
        ...
    
    def session_preparation(self) -> None:
        """Prepare the session after connection."""
        ...
    
    def terminal_settings(self) -> None:
        """Configure terminal settings."""
        ...
    
    def set_terminal_width(self, width: int) -> None:
        """
        Set terminal width.
        
        Args:
            width: Width in characters
        """
        ...
    
    def disable_paging(self) -> None:
        """Disable paging on the device."""
        ...
    
    def set_base_prompt(self) -> str:
        """
        Set and return the base prompt.
        
        Returns:
            The base prompt string
        """
        ...
    
    def save_configuration(self) -> None:
        """Save or commit the configuration."""
        ...
    
    def send_command(self, command: str) -> str:
        """
        Send a command to the device and return the output.
        
        Args:
            command: The command to execute
            
        Returns:
            The command output as a string
            
        Raises:
            ConnectionError: If the device is not connected
            CommandError: If the command execution fails
        """
        ...
    
    def send_commands(self, commands: List[str]) -> List[str]:
        """
        Send multiple commands to the device and return the outputs.
        
        Args:
            commands: List of commands to execute
            
        Returns:
            List of command outputs as strings
            
        Raises:
            ConnectionError: If the device is not connected
            CommandError: If any command execution fails
        """
        ...
    
    def send_config(self, config_commands: List[str]) -> str:
        """
        Send configuration commands to the device.
        
        Args:
            config_commands: List of configuration commands to execute
            
        Returns:
            The configuration output as a string
            
        Raises:
            ConnectionError: If the device is not connected
            CommandError: If configuration fails
        """
        ...
    
    def get_device_type(self) -> str:
        """
        Get the device type (vendor/model).
        
        Returns:
            Device type string
        """
        ...
    
    def __enter__(self) -> "PyNetworkDevice":
        """Context manager entry."""
        ...
    
    def __exit__(
        self,
        exc_type: Optional[Type[BaseException]],
        exc_val: Optional[BaseException],
        exc_tb: Optional[TracebackType]
    ) -> bool:
        """Context manager exit."""
        ...

class PyCommandResult:
    """Result of a command execution."""
    
    device_id: str
    device_type: str
    command: str
    output: str
    start_time: str
    end_time: str
    duration_ms: int
    status: str
    error: Optional[str]
    
    def to_dict(self) -> Dict[str, Any]:
        """
        Convert the command result to a dictionary.
        
        Returns:
            Dictionary representation of the command result
        """
        ...

class PyBatchCommandResults:
    """Results of batch command execution."""
    
    def get_device_results(self, device_id: str) -> Optional[List[PyCommandResult]]:
        """
        Get all results for a specific device.
        
        Args:
            device_id: The device identifier
            
        Returns:
            List of command results or None if device not found
        """
        ...
    
    def get_all_results(self) -> List[PyCommandResult]:
        """
        Get all command results.
        
        Returns:
            List of all command results
        """
        ...
    
    def get_successful_results(self) -> List[PyCommandResult]:
        """
        Get all successful command results.
        
        Returns:
            List of successful command results
        """
        ...
    
    def get_failed_results(self) -> List[PyCommandResult]:
        """
        Get all failed command results.
        
        Returns:
            List of failed command results
        """
        ...
    
    def get_command_results(self, command: str) -> List[PyCommandResult]:
        """
        Get results for a specific command.
        
        Args:
            command: The command string
            
        Returns:
            List of results for the specified command
        """
        ...
    
    def format_as_table(self) -> str:
        """
        Format results as an ASCII table.
        
        Returns:
            ASCII table as string
        """
        ...
    
    def to_json(self) -> str:
        """
        Convert results to JSON format.
        
        Returns:
            JSON string
        """
        ...
    
    def to_csv(self) -> str:
        """
        Convert results to CSV format.
        
        Returns:
            CSV string
        """
        ...
    
    def compare_outputs(self, command: str) -> Dict[str, Dict[str, List[str]]]:
        """
        Compare outputs for the same command across devices.
        
        Args:
            command: The command to compare
            
        Returns:
            Dictionary with comparison results
        """
        ...

class PyParallelExecutionManager:
    """Manager for parallel execution of commands on multiple devices."""
    
    def __init__(
        self,
        max_concurrency: Optional[int] = None,
        command_timeout_seconds: Optional[int] = None,
        connection_timeout_seconds: Optional[int] = None,
        failure_strategy: Optional[str] = None,
        reuse_connections: Optional[bool] = None
    ) -> None:
        """
        Initialize a parallel execution manager.
        
        Args:
            max_concurrency: Maximum number of concurrent connections
            command_timeout_seconds: Timeout for command execution
            connection_timeout_seconds: Timeout for connection establishment
            failure_strategy: How to handle failures ('continue' or 'abort')
            reuse_connections: Whether to reuse connections between commands
        """
        ...
    
    def set_max_concurrency(self, max_concurrency: int) -> None:
        """
        Set maximum number of concurrent connections.
        
        Args:
            max_concurrency: Maximum number of concurrent connections
        """
        ...
    
    def set_command_timeout(self, timeout_seconds: int) -> None:
        """
        Set command timeout.
        
        Args:
            timeout_seconds: Timeout in seconds
        """
        ...
    
    def set_connection_timeout(self, timeout_seconds: int) -> None:
        """
        Set connection timeout.
        
        Args:
            timeout_seconds: Timeout in seconds
        """
        ...
    
    def set_failure_strategy(self, strategy: str) -> None:
        """
        Set failure strategy.
        
        Args:
            strategy: Strategy ('continue' or 'abort')
        """
        ...
    
    def set_reuse_connections(self, reuse: bool) -> None:
        """
        Set whether to reuse connections.
        
        Args:
            reuse: True to reuse connections, False otherwise
        """
        ...
    
    def execute_command_on_all(self, devices: List[PyDeviceConfig], command: str) -> PyBatchCommandResults:
        """
        Execute a command on all devices.
        
        Args:
            devices: List of device configurations
            command: Command to execute
            
        Returns:
            Batch command results
        """
        ...
    
    def execute_commands_on_all(self, devices: List[PyDeviceConfig], commands: List[str]) -> PyBatchCommandResults:
        """
        Execute multiple commands on all devices.
        
        Args:
            devices: List of device configurations
            commands: List of commands to execute
            
        Returns:
            Batch command results
        """
        ...
    
    def execute_commands(self, device_commands: Dict[PyDeviceConfig, Union[str, List[str]]]) -> PyBatchCommandResults:
        """
        Execute specific commands on specific devices.
        
        Args:
            device_commands: Dictionary mapping devices to commands
            
        Returns:
            Batch command results
        """
        ...
    
    def cleanup(self) -> None:
        """Close all open connections."""
        ...
    
    def __enter__(self) -> "PyParallelExecutionManager":
        """Context manager entry."""
        ...
    
    def __exit__(
        self,
        exc_type: Optional[Type[BaseException]],
        exc_val: Optional[BaseException],
        exc_tb: Optional[TracebackType]
    ) -> bool:
        """Context manager exit."""
        ...