"""
Type stubs for netssh-rs Python module.

This file provides type hints for the netssh_rs module to enable proper
IntelliSense support in Python editors.
"""

from typing import Dict, List, Optional, Union, Any, Callable, Literal

class DeviceConfig:
    """Configuration for connecting to a network device."""
    
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
        ...

class CommandResult:
    """Result of a command execution on a network device."""
    
    device_id: str
    device_type: str
    command: str
    output: Optional[str]
    start_time: str
    end_time: str
    duration_ms: int
    status: str
    error: Optional[str]
    
    def is_success(self) -> bool:
        """Check if command execution was successful."""
        ...
    
    def is_failure(self) -> bool:
        """Check if command execution failed."""
        ...
    
    def is_timeout(self) -> bool:
        """Check if command execution timed out."""
        ...
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert the command result to a dictionary."""
        ...

class BatchCommandResults:
    """Collection of command results from multiple devices or commands."""
    
    def get_device_results(self, device_id: str) -> Optional[List[CommandResult]]:
        """
        Get all command results for a specific device.
        
        Args:
            device_id: Device identifier
            
        Returns:
            List of command results or None if device not found
        """
        ...
    
    def get_all_results(self) -> List[CommandResult]:
        """Get all command results."""
        ...
    
    def get_successful_results(self) -> List[CommandResult]:
        """Get all successful command results."""
        ...
    
    def get_failed_results(self) -> List[CommandResult]:
        """Get all failed command results."""
        ...
    
    def get_command_results(self, command: str) -> List[CommandResult]:
        """
        Get all results for a specific command.
        
        Args:
            command: Command string
            
        Returns:
            List of command results for the specified command
        """
        ...
    
    def format_as_table(self) -> str:
        """Format results as a table string."""
        ...
    
    def to_json(self) -> str:
        """Convert results to JSON string."""
        ...
    
    def to_csv(self) -> str:
        """Convert results to CSV string."""
        ...
    
    def compare_outputs(self, command: str) -> Dict[str, Dict[str, Union[str, bool]]]:
        """
        Compare command outputs across devices.
        
        Args:
            command: Command to compare
            
        Returns:
            Dictionary mapping devices to comparison results
        """
        ...
    
    def command_count(self) -> int:
        """Get the total number of commands executed."""
        ...
    
    def success_count(self) -> int:
        """Get the number of successful command executions."""
        ...
    
    def failure_count(self) -> int:
        """Get the number of failed command executions."""
        ...
    
    def device_count(self) -> int:
        """Get the number of devices in the results."""
        ...
    
    def duration_ms(self) -> int:
        """Get the total duration in milliseconds."""
        ...

class NetworkDevice:
    """Connection to a network device with command execution capabilities."""
    
    @staticmethod
    def create(config: DeviceConfig) -> 'NetworkDevice':
        """
        Create a new device from config.
        
        Args:
            config: The device configuration
            
        Returns:
            A new network device connection instance
        """
        ...
    
    def connect(self) -> None:
        """
        Connect to the device.
        
        Establishes an SSH connection to the network device and performs initial setup.
        
        Raises:
            ConnectionError: If connection fails
            AuthenticationError: If authentication fails
            TimeoutError: If connection times out
        """
        ...
    
    def close(self) -> None:
        """Close the connection to the device."""
        ...
    
    def check_config_mode(self) -> bool:
        """
        Check if the device is in configuration mode.
        
        Returns:
            True if device is in configuration mode, False otherwise
        """
        ...
    
    def enter_config_mode(self, config_command: Optional[str] = None) -> CommandResult:
        """
        Enter configuration mode.
        
        Args:
            config_command: Optional command to enter config mode
            
        Returns:
            Result of the command execution
        """
        ...
    
    def exit_config_mode(self, exit_command: Optional[str] = None) -> CommandResult:
        """
        Exit configuration mode.
        
        Args:
            exit_command: Optional command to exit config mode
            
        Returns:
            Result of the command execution
        """
        ...
    
    def session_preparation(self) -> None:
        """Prepare session for interaction with the device."""
        ...
    
    def terminal_settings(self) -> None:
        """Configure the terminal settings."""
        ...
    
    def set_terminal_width(self, width: int) -> None:
        """
        Set terminal width.
        
        Args:
            width: Terminal width in characters
        """
        ...
    
    def disable_paging(self) -> None:
        """Disable paging on the device."""
        ...
    
    def set_base_prompt(self) -> str:
        """
        Set the base prompt for the device.
        
        Returns:
            The detected base prompt
        """
        ...
    
    def save_configuration(self) -> CommandResult:
        """
        Save the device configuration.
        
        Returns:
            Result of the command execution
        """
        ...
    
    def send_command(
        self,
        command: str,
        expect_string: Optional[str] = None,
        read_timeout: Optional[float] = None,
        auto_find_prompt: Optional[bool] = None,
        strip_prompt: Optional[bool] = None,
        strip_command: Optional[bool] = None,
        normalize: Optional[bool] = None,
        cmd_verify: Optional[bool] = None
    ) -> CommandResult:
        """
        Send a command to the device.
        
        Args:
            command: Command to execute
            expect_string: String to expect in output
            read_timeout: Timeout for reading output
            auto_find_prompt: Whether to auto-find the prompt
            strip_prompt: Whether to strip the prompt from output
            strip_command: Whether to strip the command from output
            normalize: Whether to normalize the output
            cmd_verify: Whether to verify the command
            
        Returns:
            Result of the command execution
        """
        ...
    
    def send_command_simple(self, command: str) -> CommandResult:
        """
        Send a simple command to the device.
        
        Args:
            command: Command to execute
            
        Returns:
            Result of the command execution
        """
        ...
    
    def get_device_type(self) -> str:
        """
        Get the device type.
        
        Returns:
            Device type string
        """
        ...
    
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
            config_commands: List of configuration commands
            exit_config_mode: Whether to exit config mode after execution
            read_timeout: Timeout for reading output
            strip_prompt: Whether to strip the prompt from output
            strip_command: Whether to strip the command from output
            config_mode_command: Command to enter config mode
            cmd_verify: Whether to verify commands
            enter_config_mode: Whether to enter config mode
            error_pattern: Pattern to identify errors
            terminator: Command terminator
            bypass_commands: Commands to bypass
            fast_cli: Whether to use fast CLI mode
            
        Returns:
            Result of the configuration commands execution
        """
        ...

class ParallelExecutionManager:
    """Manager for parallel execution of commands on multiple devices."""
    
    def __init__(
        self,
        max_concurrency: Optional[int] = None,
        command_timeout_seconds: Optional[int] = None,
        connection_timeout_seconds: Optional[int] = None,
        failure_strategy: Optional[Literal["continue", "fast_fail", "retry"]] = None,
        reuse_connections: Optional[bool] = None
    ) -> None:
        """
        Initialize a parallel execution manager.
        
        Args:
            max_concurrency: Maximum number of concurrent connections
            command_timeout_seconds: Timeout for command execution
            connection_timeout_seconds: Timeout for connection establishment
            failure_strategy: Strategy for handling failures
            reuse_connections: Whether to reuse connections
        """
        ...
    
    def set_max_concurrency(self, max_concurrency: int) -> None:
        """
        Set the maximum number of concurrent connections.
        
        Args:
            max_concurrency: Maximum number of concurrent connections
        """
        ...
    
    def set_command_timeout(self, timeout_seconds: int) -> None:
        """
        Set the command execution timeout.
        
        Args:
            timeout_seconds: Timeout in seconds
        """
        ...
    
    def set_connection_timeout(self, timeout_seconds: int) -> None:
        """
        Set the connection timeout.
        
        Args:
            timeout_seconds: Timeout in seconds
        """
        ...
    
    def set_failure_strategy(self, strategy: Literal["continue", "fast_fail", "retry"]) -> None:
        """
        Set the failure handling strategy.
        
        Args:
            strategy: Strategy for handling failures
        """
        ...
    
    def set_reuse_connections(self, reuse: bool) -> None:
        """
        Set whether to reuse connections.
        
        Args:
            reuse: Whether to reuse connections
        """
        ...
    
    def execute_command_on_all(
        self,
        configs: List[DeviceConfig],
        command: str
    ) -> BatchCommandResults:
        """
        Execute a single command on multiple devices.
        
        Args:
            configs: List of device configurations
            command: Command to execute
            
        Returns:
            Results of command execution
        """
        ...
    
    def execute_commands_on_all(
        self,
        configs: List[DeviceConfig],
        commands: List[str]
    ) -> BatchCommandResults:
        """
        Execute multiple commands on multiple devices.
        
        Args:
            configs: List of device configurations
            commands: List of commands to execute
            
        Returns:
            Results of command execution
        """
        ...
    
    def execute_commands(
        self,
        device_commands: Dict[str, List[str]]
    ) -> BatchCommandResults:
        """
        Execute different commands on different devices.
        
        Args:
            device_commands: Dictionary mapping device hostnames to commands
            
        Returns:
            Results of command execution
        """
        ...
    
    def cleanup(self) -> None:
        """Clean up resources."""
        ...

def initialize_logging(
    level: str = "info",
    log_to_file: bool = False,
    log_file_path: Optional[str] = None,
    log_format: Optional[str] = None
) -> None:
    """
    Initialize logging for the netssh-rs library.
    
    Args:
        level: Log level (error, warn, info, debug, trace)
        log_to_file: Whether to log to a file
        log_file_path: Path to log file (if log_to_file is True)
        log_format: Custom log format string
    """
    ...

def set_default_session_logging(
    enable: bool = False,
    log_path: Optional[str] = None
) -> None:
    """
    Set default session logging behavior globally.

    This function configures whether session logging is enabled by default
    and where the logs are stored when no specific path is provided.
    This setting applies to all device connections that don't explicitly
    specify a session_log parameter.

    Args:
        enable: Whether to enable session logging by default. Default is False.
        log_path: The directory path where session logs will be stored.
                 If None, the existing path setting will be maintained.
    """
    ...

# Define TextFSM functions directly in this file
def parse_output(platform: str, command: str, data: str) -> List[Dict[str, str]]:
    """
    Parse command output using TextFSM.
    
    Args:
        platform: Device platform (e.g., cisco_ios, cisco_nxos)
        command: Command that was executed
        data: Command output to parse
    
    Returns:
        List of dictionaries, each representing a row of parsed data
    """
    ...

def parse_output_to_json(platform: str, command: str, data: str) -> str:
    """
    Parse command output using TextFSM and return as JSON.
    
    Args:
        platform: Device platform (e.g., cisco_ios, cisco_nxos)
        command: Command that was executed
        data: Command output to parse
    
    Returns:
        JSON string representing the parsed data
    """
    ...

class NetworkOutputParser:
    """Class for parsing network device command outputs using TextFSM templates."""
    
    template_dir: str
    
    def __init__(self, template_dir: Optional[str] = None) -> None:
        """
        Initialize the parser with a template directory.
        
        Args:
            template_dir: Optional path to template directory. If not provided,
                         uses the default template directory.
        """
        ...
    
    def find_template(self, platform: str, command: str) -> Optional[str]:
        """
        Find the appropriate template file for a platform and command.
        
        Args:
            platform: Device platform (e.g., cisco_ios, cisco_nxos)
            command: Command that was executed
        
        Returns:
            Path to the template file, or None if not found
        """
        ...
    
    def parse_output(self, platform: str, command: str, data: str) -> List[Dict[str, str]]:
        """
        Parse command output using TextFSM.
        
        Args:
            platform: Device platform (e.g., cisco_ios, cisco_nxos)
            command: Command that was executed
            data: Command output to parse
        
        Returns:
            List of dictionaries, each representing a row of parsed data
        """
        ...
    
    def parse_to_json(self, platform: str, command: str, data: str) -> str:
        """
        Parse command output using TextFSM and return as JSON.
        
        Args:
            platform: Device platform (e.g., cisco_ios, cisco_nxos)
            command: Command that was executed
            data: Command output to parse
        
        Returns:
            JSON string representing the parsed data
        """
        ... 