from typing import Dict, List, Optional, Union, Any
from datetime import datetime
import os

def initialize_logging(debug: bool = False, console: bool = False) -> None:
    """
    Initialize logging for the library.
    
    Args:
        debug: Enable debug logging
        console: Enable console output
    """
    pass

class DeviceConfig:
    """
    Configuration for connecting to a network device.
    
    Contains all the necessary information to establish an SSH connection.
    """
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
        Create a new device configuration.
        
        Args:
            device_type: The type of device (e.g., 'cisco_ios', 'juniper')
            host: The hostname or IP address of the device
            username: The username for authentication
            password: The password for authentication (optional)
            port: The SSH port (default: 22)
            timeout_seconds: Connection timeout in seconds (default: 60)
            secret: The enable secret for privileged mode (if required)
            session_log: Path to log file for session logging (optional)
        """
        pass

class DeviceInfo:
    """Information about a connected network device."""
    device_type: str
    model: str
    version: str
    hostname: str
    serial: str
    uptime: str

class CommandResult:
    """
    Result of executing a command on a network device.
    
    Contains detailed information about the command execution, including:
    - device_id: The identifier for the device
    - device_type: The type of device
    - command: The command that was executed
    - output: The output text from the command
    - start_time: When command execution started
    - end_time: When command execution ended
    - duration_ms: How long the command took to execute in milliseconds
    - status: The execution status (Success, Failed, Timeout, Skipped)
    - error: Error message if the command failed
    """
    
    device_id: str
    device_type: str
    command: str
    output: Optional[str]
    start_time: str
    end_time: str
    duration_ms: int
    status: str
    error: Optional[str]
    
    def to_dict(self) -> Dict[str, Any]:
        """
        Convert the command result to a Python dictionary.
        
        Returns:
            Dictionary representation of the command result
        """
        pass
    
    def is_success(self) -> bool:
        """
        Check if the command was successful.
        
        Returns:
            True if command executed successfully, False otherwise
        """
        pass
    
    def is_failure(self) -> bool:
        """
        Check if the command failed.
        
        Returns:
            True if command failed, False otherwise
        """
        pass
    
    def is_timeout(self) -> bool:
        """
        Check if the command timed out.
        
        Returns:
            True if command timed out, False otherwise
        """
        pass

class BatchCommandResults:
    """
    Results of executing commands on multiple devices.
    
    Provides methods to access and analyze the results in various formats.
    """
    
    command_count: int
    success_count: int
    failure_count: int
    device_count: int
    duration_ms: int
    
    def get_device_results(self, device_id: str) -> Optional[List[CommandResult]]:
        """
        Get all results for a specific device.
        
        Args:
            device_id: The device identifier
            
        Returns:
            A list of CommandResult objects for the specified device, or None if device not found
        """
        pass
    
    def get_all_results(self) -> List[CommandResult]:
        """
        Get all command results across all devices.
        
        Returns:
            A list of all CommandResult objects
        """
        pass
    
    def get_successful_results(self) -> List[CommandResult]:
        """
        Get all successful command results.
        
        Returns:
            A list of CommandResult objects with Success status
        """
        pass
    
    def get_failed_results(self) -> List[CommandResult]:
        """
        Get all failed command results.
        
        Returns:
            A list of CommandResult objects with Failed status
        """
        pass
    
    def get_command_results(self, command: str) -> List[CommandResult]:
        """
        Get results for a specific command across all devices.
        
        Args:
            command: The command to filter by
            
        Returns:
            A list of CommandResult objects for the specified command
        """
        pass
    
    def format_as_table(self) -> str:
        """
        Format the results as a table for display.
        
        Returns:
            A formatted string containing a table of results
        """
        pass
    
    def to_json(self) -> str:
        """
        Convert the batch results to JSON.
        
        Returns:
            A JSON string representation of the results
        """
        pass
    
    def to_csv(self) -> str:
        """
        Convert the batch results to CSV.
        
        Returns:
            A CSV string representation of the results
        """
        pass
    
    def compare_outputs(self, command: str) -> Dict[str, str]:
        """
        Compare command outputs across devices.
        
        Args:
            command: The command to compare across devices
            
        Returns:
            A dictionary mapping devices to their command outputs
        """
        pass

class NetworkDevice:
    """
    Connection to a network device.
    
    Provides methods for sending commands, managing configuration mode, and more.
    """
    
    @staticmethod
    def create(config: DeviceConfig) -> 'NetworkDevice':
        """
        Create a new device from config.
        
        Creates a new network device connection handler based on the provided configuration.
        
        Args:
            config: The device configuration
            
        Returns:
            A new NetworkDevice instance
            
        Raises:
            RuntimeError: If device creation fails
        """
        pass
    
    def connect(self) -> None:
        """
        Connect to the device.
        
        Establishes an SSH connection to the network device and performs initial setup.
        
        Raises:
            ConnectionError: If connection fails
            AuthenticationError: If authentication fails
            TimeoutError: If connection times out
        """
        pass
    
    def close(self) -> None:
        """
        Close the connection to the device.
        """
        pass
    
    def check_config_mode(self) -> bool:
        """
        Check if the device is in configuration mode.
        
        Returns:
            True if device is in configuration mode, False otherwise
        """
        pass
    
    def enter_config_mode(self, config_command: Optional[str] = None) -> CommandResult:
        """
        Enter configuration mode.
        
        Args:
            config_command: Custom configuration command to use
            
        Returns:
            Result of the command execution
        """
        pass
    
    def exit_config_mode(self, exit_command: Optional[str] = None) -> CommandResult:
        """
        Exit configuration mode.
        
        Args:
            exit_command: Custom exit command to use
            
        Returns:
            Result of the command execution
        """
        pass
    
    def session_preparation(self) -> None:
        """
        Prepare the session after connection.
        """
        pass
    
    def terminal_settings(self) -> None:
        """
        Configure terminal settings.
        """
        pass
    
    def set_terminal_width(self, width: int) -> None:
        """
        Set terminal width.
        
        Args:
            width: Terminal width in characters
        """
        pass
    
    def disable_paging(self) -> None:
        """
        Disable paging on the device.
        """
        pass
    
    def set_base_prompt(self) -> str:
        """
        Set the base prompt.
        
        Returns:
            The detected base prompt
        """
        pass
    
    def save_configuration(self) -> CommandResult:
        """
        Save the device configuration.
        
        Returns:
            Result of the save configuration command
        """
        pass
    
    def send_command(self, command: str) -> CommandResult:
        """
        Send a command to the device.
        
        Args:
            command: The command to execute on the device
            
        Returns:
            Result of the command execution containing output, timing information, and status
        """
        pass
    
    def get_device_type(self) -> str:
        """
        Get the device type.
        
        Returns:
            The device type string
        """
        pass
    
    def send_config_set(
        self,
        config_commands: List[str],
        exit_config_mode: Optional[bool] = None,
        read_timeout: Optional[float] = None,
        strip_prompt: Optional[bool] = None,
        strip_command: Optional[bool] = None,
        config_mode_command: Optional[str] = None,
        cmd_verify: Optional[bool] = None,
        enter_config_mode: Optional[bool] = None,
        error_pattern: Optional[str] = None,
        terminator: Optional[str] = None,
        bypass_commands: Optional[str] = None,
        fast_cli: Optional[bool] = None
    ) -> CommandResult:
        """
        Send configuration commands to the device.
        
        Args:
            config_commands: List of configuration commands to send
            exit_config_mode: Whether to exit config mode after sending commands
            read_timeout: Read timeout in seconds
            strip_prompt: Whether to strip the prompt from the output
            strip_command: Whether to strip the command from the output
            config_mode_command: Custom command to enter config mode
            cmd_verify: Whether to verify command echoing
            enter_config_mode: Whether to enter config mode before sending commands
            error_pattern: Regex pattern to detect errors in output
            terminator: Command terminator character
            bypass_commands: Commands to bypass verification
            fast_cli: Whether to optimize for faster command execution
            
        Returns:
            Result of the config commands execution
        """
        pass
    
    def __enter__(self) -> 'NetworkDevice':
        """Context manager support - enter"""
        return self
    
    def __exit__(self, exc_type: Any, exc_value: Any, traceback: Any) -> bool:
        """Context manager support - exit"""
        return False

class ParallelExecutionManager:
    """
    Manager for parallel execution of commands on multiple devices.
    
    Provides methods for executing commands in parallel on multiple devices with
    configurable concurrency, timeout, and failure handling strategies.
    """
    
    def __init__(
        self,
        max_concurrency: Optional[int] = None,
        command_timeout_seconds: Optional[int] = None,
        connection_timeout_seconds: Optional[int] = None,
        failure_strategy: Optional[str] = None,
        reuse_connections: Optional[bool] = None
    ) -> None:
        """
        Create a new ParallelExecutionManager.
        
        Args:
            max_concurrency: Maximum number of concurrent connections
            command_timeout_seconds: Command timeout in seconds
            connection_timeout_seconds: Connection timeout in seconds
            failure_strategy: Strategy for handling failures ('continue_on_device', 'skip_device', 'abort_batch')
            reuse_connections: Whether to reuse connections between command executions
        """
        pass
    
    def set_max_concurrency(self, max_concurrency: int) -> None:
        """
        Set the maximum concurrency.
        
        Args:
            max_concurrency: Maximum number of concurrent connections
        """
        pass
    
    def set_command_timeout(self, timeout_seconds: int) -> None:
        """
        Set the command timeout.
        
        Args:
            timeout_seconds: Command timeout in seconds
        """
        pass
    
    def set_connection_timeout(self, timeout_seconds: int) -> None:
        """
        Set the connection timeout.
        
        Args:
            timeout_seconds: Connection timeout in seconds
        """
        pass
    
    def set_failure_strategy(self, strategy: str) -> None:
        """
        Set the failure strategy.
        
        Args:
            strategy: Strategy for handling failures ('continue_on_device', 'skip_device', 'abort_batch')
        """
        pass
    
    def set_reuse_connections(self, reuse: bool) -> None:
        """
        Set whether to reuse connections.
        
        Args:
            reuse: Whether to reuse connections between command executions
        """
        pass
    
    def execute_command_on_all(
        self,
        configs: List[DeviceConfig],
        command: str
    ) -> BatchCommandResults:
        """
        Execute a command on all devices.
        
        Args:
            configs: List of device configurations
            command: Command to execute on all devices
            
        Returns:
            Results of the command execution on all devices
        """
        pass
    
    def execute_commands_on_all(
        self,
        configs: List[DeviceConfig],
        commands: List[str]
    ) -> BatchCommandResults:
        """
        Execute multiple commands sequentially on all devices in parallel.
        
        Args:
            configs: List of device configurations
            commands: List of commands to execute on all devices
            
        Returns:
            Results of the commands execution on all devices
        """
        pass
    
    def execute_commands(
        self,
        device_commands: Dict[DeviceConfig, List[str]]
    ) -> BatchCommandResults:
        """
        Execute different commands on different devices.
        
        Args:
            device_commands: Dictionary mapping device configurations to lists of commands
            
        Returns:
            Results of the commands execution on all devices
        """
        pass
    
    def cleanup(self) -> None:
        """
        Clean up all connections.
        """
        pass
    
    def __enter__(self) -> 'ParallelExecutionManager':
        """Context manager support - enter"""
        return self
    
    def __exit__(self, exc_type: Any, exc_value: Any, traceback: Any) -> bool:
        """Context manager support - exit"""
        return False 