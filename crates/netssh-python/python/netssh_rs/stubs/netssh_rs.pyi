"""
Type stubs for the netssh_rs Python module.

This module provides type annotations for the Rust-implemented netssh_rs library,
which offers high-performance network device automation via SSH.

The main components include:
- Device configuration (PyDeviceConfig)
- Network device connection and command execution (PyNetworkDevice)
- Command results and output handling (PyCommandResult, PyBatchCommandResults)
- Parallel execution across multiple devices (PyParallelExecutionManager)
- TextFSM parsing for structured command output (NetworkOutputParser)

These type stubs are for IDE integration and type checking and are not used at runtime.
"""

from typing import Any, Callable, Dict, List, Optional, Tuple, TypeVar, Union, overload, Type
from types import TracebackType
import datetime

T = TypeVar('T', bound='PyNetworkDevice')

__version__: str
"""The version of the netssh_rs package"""

# Re-exports for convenience
NetworkDevice = PyNetworkDevice
DeviceConfig = PyDeviceConfig
CommandResult = PyCommandResult
BatchCommandResults = PyBatchCommandResults
ParallelExecutionManager = PyParallelExecutionManager
ParallelExecutionConfig = PyParallelExecutionConfig

# TextFSM Parser support
class NetworkOutputParser:
    """
    Class for parsing network device command outputs using TextFSM templates.
    
    This parser uses TextFSM templates to convert unstructured command output 
    into structured data formats.
    """
    
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
        Find the appropriate template for a given platform and command.
        
        Args:
            platform: Device platform (e.g., cisco_ios)
            command: Command string (e.g., show version)
            
        Returns:
            Path to template file or None if not found
        """
        ...
    
    def parse_output(self, platform: str, command: str, data: str) -> Optional[List[Dict[str, str]]]:
        """
        Parse command output using TextFSM.
        
        Args:
            platform: Device platform (e.g., cisco_ios)
            command: Command string (e.g., show version)
            data: Command output as string
        
        Returns:
            List of dictionaries containing parsed data, or None if parsing fails
        """
        ...
    
    def parse_to_json(self, platform: str, command: str, data: str) -> Optional[str]:
        """
        Parse command output and return as JSON string.
        
        Args:
            platform: Device platform (e.g., cisco_ios)
            command: Command string (e.g., show version)
            data: Command output as string
            
        Returns:
            JSON string of parsed data, or None if parsing fails
        """
        ...

def parse_output(platform: str, command: str, data: str) -> Optional[List[Dict[str, str]]]:
    """
    Parse command output using TextFSM.
    
    Helper function that creates a NetworkOutputParser instance and calls parse_output.
    
    Args:
        platform: Device platform (e.g., cisco_ios)
        command: Command string (e.g., show version)
        data: Command output as string
        
    Returns:
        List of dictionaries containing parsed data, or None if parsing fails
    """
    ...

def parse_output_to_json(platform: str, command: str, data: str) -> Optional[str]:
    """
    Parse command output and return as JSON string.
    
    Helper function that creates a NetworkOutputParser instance and calls parse_to_json.
    
    Args:
        platform: Device platform (e.g., cisco_ios)
        command: Command string (e.g., show version)
        data: Command output as string
        
    Returns:
        JSON string of parsed data, or None if parsing fails
    """
    ...

# Specific device creation helpers
def create_cisco_ios_device(
    hostname: str,
    username: str,
    password: str,
    port: int = 22,
    enable_password: Optional[str] = None,
    **kwargs: Any
) -> PyNetworkDevice:
    """
    Create a Cisco IOS device with the specified configuration.

    This is a convenience function that creates a PyDeviceConfig with the
    device_type set to 'cisco_ios' and then creates a PyNetworkDevice.

    Args:
        hostname: The hostname or IP address of the device
        username: The username for authentication
        password: The password for authentication
        port: The SSH port to connect to (default 22)
        enable_password: The enable password for privileged mode (optional)
        **kwargs: Additional configuration options to pass to PyDeviceConfig

    Returns:
        A configured PyNetworkDevice instance ready to connect

    Examples:
        ```python
        # Create a Cisco IOS device
        device = create_cisco_ios_device(
            hostname="192.168.1.1",
            username="admin",
            password="password",
            enable_password="enable_pass"
        )
        
        # Connect and send a command
        with device:
            result = device.send_command("show version")
            print(result.output)
        ```
    """
    ...

def create_juniper_junos_device(
    hostname: str,
    username: str,
    password: str,
    port: int = 22,
    **kwargs: Any
) -> PyNetworkDevice:
    """
    Create a Juniper JunOS device with the specified configuration.

    This is a convenience function that creates a PyDeviceConfig with the
    device_type set to 'juniper_junos' and then creates a PyNetworkDevice.

    Args:
        hostname: The hostname or IP address of the device
        username: The username for authentication
        password: The password for authentication
        port: The SSH port to connect to (default 22)
        **kwargs: Additional configuration options to pass to PyDeviceConfig

    Returns:
        A configured PyNetworkDevice instance ready to connect

    Examples:
        ```python
        # Create a Juniper JunOS device
        device = create_juniper_junos_device(
            hostname="192.168.1.2",
            username="admin",
            password="password"
        )
        
        # Connect and send a command
        with device:
            result = device.send_command("show version")
            print(result.output)
        ```
    """
    ...

def create_arista_eos_device(
    hostname: str,
    username: str,
    password: str,
    port: int = 22,
    enable_password: Optional[str] = None,
    **kwargs: Any
) -> PyNetworkDevice:
    """
    Create an Arista EOS device with the specified configuration.

    This is a convenience function that creates a PyDeviceConfig with the
    device_type set to 'arista_eos' and then creates a PyNetworkDevice.

    Args:
        hostname: The hostname or IP address of the device
        username: The username for authentication
        password: The password for authentication
        port: The SSH port to connect to (default 22)
        enable_password: The enable password for privileged mode (optional)
        **kwargs: Additional configuration options to pass to PyDeviceConfig

    Returns:
        A configured PyNetworkDevice instance ready to connect

    Examples:
        ```python
        # Create an Arista EOS device
        device = create_arista_eos_device(
            hostname="192.168.1.3",
            username="admin",
            password="password"
        )
        
        # Connect and send a command
        with device:
            result = device.send_command("show version")
            print(result.output)
        ```
    """
    ...

# Constants
SUPPORTED_DEVICE_TYPES: List[str]
"""List of supported device types in the library"""

def initialize_logging(
    level: str = "info",
    log_to_file: bool = False,
    log_file_path: Optional[str] = None,
    log_format: Optional[str] = None
) -> None:
    """
    Initialize the logging system for the netssh-rs library.

    This function configures the logging system for the netssh-rs library.
    It must be called before any other functions in the library are used
    if you want to capture log output.

    Args:
        level: The log level to use. One of "error", "warn", "info", "debug", "trace".
               Default is "info".
        log_to_file: Whether to log to a file. Default is False.
        log_file_path: The path to the log file. Only used if log_to_file is True.
                       If None, a default path will be used.
        log_format: The format string to use for log messages. If None, a default format
                   will be used. See the documentation for the log crate for details on
                   the format string syntax.

    Examples:
        ```python
        # Initialize logging with default settings (info level, console only)
        initialize_logging()
        
        # Initialize logging with debug level
        initialize_logging(level="debug")
        
        # Initialize logging with output to a file
        initialize_logging(
            level="debug",
            log_to_file=True,
            log_file_path="/var/log/netssh-rs.log"
        )
        ```
    """
    ...

class PyDeviceConfig:
    """
    Configuration for a network device connection.

    This class contains all the parameters needed to establish an SSH connection
    to a network device. It acts as the configuration object for PyNetworkDevice.

    Attributes:
        device_type: The device's operating system type (e.g., 'cisco_ios', 'juniper_junos')
        host: The IP address or hostname of the device
        username: SSH username for authentication
        password: SSH password for authentication
        port: SSH port number (default: 22)
        timeout_seconds: Connection timeout in seconds (default: 60)
        secret: Enable secret for privileged mode access
        session_log: File path for logging the SSH session

    Examples:
        ```python
        # Create a basic Cisco IOS device configuration
        cisco_config = PyDeviceConfig(
            device_type="cisco_ios",
            host="192.168.1.1",
            username="admin",
            password="cisco"
        )

        # Create a configuration with additional parameters
        juniper_config = PyDeviceConfig(
            device_type="juniper_junos",
            host="router.example.com",
            username="admin",
            password="juniper",
            port=2222,
            timeout_seconds=120,
            session_log="/tmp/session.log"
        )
        ```
    """

    device_type: str
    """The type of device OS (e.g., 'cisco_ios', 'juniper_junos', 'arista_eos')"""

    host: str
    """The hostname or IP address of the device"""

    username: str
    """The username for SSH authentication"""

    password: Optional[str]
    """The password for SSH authentication (can be None if using key-based auth)"""

    port: Optional[int]
    """The SSH port (default: 22)"""

    timeout_seconds: Optional[int]
    """Connection timeout in seconds (default: 60)"""

    secret: Optional[str]
    """The enable secret for privileged mode access (used in Cisco devices)"""

    session_log: Optional[str]
    """Path to log file for detailed session logging"""

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

        Creates a new device configuration with the specified parameters.

        Args:
            device_type: The device's operating system type (e.g., 'cisco_ios', 'juniper_junos')
                Supported types include: 'cisco_ios', 'cisco_nxos', 'juniper_junos',
                'arista_eos', 'cisco_xr', 'huawei', 'fortinet'
            host: The IP address or hostname of the device to connect to
            username: SSH username for authentication
            password: SSH password for authentication (can be None if using key-based authentication)
            port: SSH port number (default: 22)
            timeout_seconds: Connection timeout in seconds (default: 60)
            secret: Enable secret for privileged mode access (primarily used for Cisco devices)
            session_log: File path for logging the entire SSH session

        Examples:
            ```python
            # Simple configuration for Cisco device
            config = PyDeviceConfig(
                device_type="cisco_ios",
                host="192.168.1.1",
                username="admin",
                password="cisco123"
            )
            ```
        """
        ...

class PyDeviceInfo:
    """
    Information about a network device.

    This class contains detailed information about the network device that
    is collected during connection. It provides hardware and software details
    that can be useful for inventory management and troubleshooting.

    Attributes:
        vendor: The device manufacturer (e.g., 'Cisco', 'Juniper', 'Arista')
        model: The specific hardware model number
        os_version: The operating system version or firmware version
        hostname: The configured hostname of the device
        uptime: The device uptime as a formatted string

    Examples:
        ```python
        # Get device info from a connected device
        device = PyNetworkDevice.create(config)
        device.connect()
        info = device.get_device_info()

        print(f"Device: {info.hostname}")
        print(f"Model: {info.model} running {info.os_version}")
        print(f"Uptime: {info.uptime}")
        ```
    """

    vendor: str
    """The device manufacturer (e.g., 'Cisco', 'Juniper', 'Arista')"""

    model: str
    """The specific hardware model number (e.g., 'ISR4331', 'MX240', 'DCS-7280R')"""

    os_version: str
    """The operating system version (e.g., 'IOS 15.2(4)M', 'Junos 20.4R1.12')"""

    hostname: str
    """The configured hostname of the device as reported by the device itself"""

    uptime: str
    """The device uptime as a formatted string (e.g., '10 days, 23 hours, 15 minutes')"""

class PyNetworkDevice:
    """
    Network device connection handler.

    This class manages all aspects of an SSH connection to a network device,
    including connecting, sending commands, and handling configuration mode.
    It provides methods to interact with the device at various privilege levels.

    The PyNetworkDevice should be created using the static `create` method and
    properly closed when no longer needed, preferably using a context manager.

    Examples:
        ```python
        # Basic usage
        config = PyDeviceConfig(
            device_type="cisco_ios",
            host="192.168.1.1",
            username="admin",
            password="cisco123"
        )

        # Create and use a device
        device = PyNetworkDevice.create(config)
        device.connect()
        output = device.send_command("show version")
        device.close()

        # Using context manager (recommended)
        with PyNetworkDevice.create(config) as device:
            output = device.send_command("show version")
            configs = device.send_commands(["show run", "show ip int brief"])
        ```
    """

    @classmethod
    def create(cls: Type[T], config: PyDeviceConfig) -> T:
        """
        Create a new network device connection.

        Factory method that creates a new PyNetworkDevice instance from the provided
        configuration. This is the recommended way to create a device object.

        Args:
            config: A PyDeviceConfig object containing connection parameters

        Returns:
            A new PyNetworkDevice instance ready to connect

        Raises:
            RuntimeError: If device creation fails due to invalid configuration

        Examples:
            ```python
            config = PyDeviceConfig(
                device_type="cisco_ios",
                host="192.168.1.1",
                username="admin",
                password="cisco123"
            )
            device = PyNetworkDevice.create(config)
            ```
        """
        ...

    def connect(self) -> None:
        """
        Connect to the network device.

        Establishes an SSH connection to the device using the configuration
        provided when the PyNetworkDevice was created. This method handles
        authentication, initial terminal setup, and privilege level.

        Raises:
            RuntimeError: If connection fails (auth failure, network unreachable, etc.)

        Examples:
            ```python
            device = PyNetworkDevice.create(config)
            try:
                device.connect()
                # Device is now connected
            except RuntimeError as e:
                print(f"Connection failed: {e}")
            ```
        """
        ...

    def close(self) -> None:
        """
        Close the connection to the device.

        Properly terminates the SSH session. This method should always be called
        when finished with the device to free resources, or use the context manager.

        Raises:
            RuntimeError: If disconnection fails unexpectedly

        Examples:
            ```python
            device = PyNetworkDevice.create(config)
            device.connect()
            # Do operations...
            device.close()  # Properly close the connection
            ```
        """
        ...

    def check_config_mode(self) -> bool:
        """
        Check if the device is in configuration mode.

        Tests whether the current session is in configuration mode by examining
        the command prompt.

        Returns:
            True if in configuration mode, False if in user/enable mode

        Raises:
            RuntimeError: If the check cannot be performed (e.g., disconnected)

        Examples:
            ```python
            if not device.check_config_mode():
                device.enter_config_mode()
            # Now in config mode
            ```
        """
        ...

    def enter_config_mode(self, config_command: Optional[str] = None) -> PyCommandResult:
        """
        Enter configuration mode.

        Switches the device session to configuration mode, where changes to the
        device configuration can be made. For most devices, this is equivalent
        to the "configure terminal" command.

        Args:
            config_command: Optional custom configuration command if the default
                            command for the device type is not appropriate

        Returns:
            CommandResult: Result of the command execution

        Raises:
            RuntimeError: If entering config mode fails (e.g., permission denied)

        Examples:
            ```python
            # Enter config mode with default command
            result = device.enter_config_mode()
            
            # Use custom config command
            result = device.enter_config_mode(config_command="conf t")
            ```
        """
        ...

    def exit_config_mode(self, exit_command: Optional[str] = None) -> PyCommandResult:
        """
        Exit configuration mode.

        Returns from configuration mode to enable/privileged mode. For most
        devices, this is equivalent to the "end" or "exit" command.

        Args:
            exit_command: Optional custom exit command if the default command
                        for the device type is not appropriate

        Returns:
            CommandResult: Result of the command execution

        Raises:
            RuntimeError: If exiting config mode fails

        Examples:
            ```python
            # Exit config mode with default command
            result = device.exit_config_mode()

            # Use custom exit command
            result = device.exit_config_mode(exit_command="end")
            ```
        """
        ...

    def session_preparation(self) -> None:
        """
        Prepare the session after connection.

        Performs initial setup tasks after establishing a connection, such as
        setting terminal width, disabling paging, and determining the base prompt.
        This is typically called automatically by the connect() method.

        Raises:
            RuntimeError: If session preparation fails

        Examples:
            ```python
            device.connect()
            # Session is automatically prepared

            # If needed manually:
            device.session_preparation()
            ```
        """
        ...

    def terminal_settings(self) -> None:
        """
        Configure terminal settings.

        Sets up terminal behavior for optimal interaction, such as disabling
        line wrap and setting width. This is typically called automatically
        by session_preparation().

        Raises:
            RuntimeError: If terminal configuration fails

        Examples:
            ```python
            # Usually not called directly, but if needed:
            device.terminal_settings()
            ```
        """
        ...

    def save_configuration(self) -> PyCommandResult:
        """
        Save or commit the configuration.

        Persists configuration changes using the appropriate command for the
        device type (e.g., "write memory" for Cisco IOS, "commit" for Juniper).

        Returns:
            CommandResult: Result of the save configuration command

        Raises:
            RuntimeError: If saving configuration fails

        Examples:
            ```python
            # Make config changes
            device.enter_config_mode()
            device.send_command("interface GigabitEthernet0/1")
            device.send_command("description WAN Link")
            device.exit_config_mode()

            # Save changes
            result = device.save_configuration()
            ```
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
    ) -> PyCommandResult:
        """
        Send a command to the device and return the result.

        Executes a single command on the device and waits for completion.
        This method handles command termination detection and timing.

        Args:
            command: The command string to execute on the device
            expect_string: Optional pattern to search for in the output
            read_timeout: Optional timeout in seconds for reading output
            auto_find_prompt: Optional flag to automatically find prompt
            strip_prompt: Optional flag to strip prompt from output
            strip_command: Optional flag to strip command from output
            normalize: Optional flag to normalize line feeds
            cmd_verify: Optional flag to verify command echoing

        Returns:
            PyCommandResult: Command result containing output and status information

        Raises:
            RuntimeError: If command execution fails or times out

        Examples:
            ```python
            # Get interface status
            result = device.send_command("show ip interface brief")
            
            # Check if the command was successful
            if result.is_success():
                print(f"Output: {result.output}")
            else:
                print(f"Error: {result.error}")
            ```
        """
        ...

    def get_device_type(self) -> str:
        """
        Get the device type.

        Returns:
            Device type string

        Raises:
            RuntimeError: If getting device type fails
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
    ) -> PyCommandResult:
        """
        Send a set of configuration commands to the device.

        This method provides a flexible way to send configuration commands with various
        options for verification, error handling, and output processing. It's particularly
        useful for making configuration changes to network devices.

        Args:
            config_commands: List of configuration commands to send to the device
            exit_config_mode: Whether to exit config mode after sending commands (default: True)
            read_timeout: Timeout for reading command output in seconds (default: 15.0)
            strip_prompt: Whether to strip the prompt from the output (default: False)
            strip_command: Whether to strip the command from the output (default: False)
            config_mode_command: Custom command to enter config mode (device-specific default if None)
            cmd_verify: Whether to verify each command was accepted (default: True)
            enter_config_mode: Whether to enter config mode before sending commands (default: True)
            error_pattern: Regex pattern to detect configuration errors (default: None)
            terminator: Alternate terminator pattern for command completion (default: '#')
            bypass_commands: Regex pattern for commands that should bypass verification (default: 'banner .*')
            fast_cli: Whether to use fast mode with minimal verification (default: False)

        Returns:
            PyCommandResult: Result of the command execution

        Raises:
            RuntimeError: If configuration fails or device rejects commands

        Examples:
            ```python
            # Basic configuration example
            result = device.send_config_set([
                "interface GigabitEthernet0/1",
                "description WAN Link",
                "ip address 192.168.1.1 255.255.255.0",
                "no shutdown"
            ])

            # Advanced configuration with custom options
            result = device.send_config_set(
                config_commands=[
                    "router ospf 1",
                    "network 192.168.1.0 0.0.0.255 area 0"
                ],
                error_pattern=r"% Invalid input",
                cmd_verify=True,
                read_timeout=30.0
            )
            ```
        """
        ...

    def execute_command(
        self,
        command: str,
        expect_string: Optional[str] = None,
        read_timeout: Optional[float] = None,
        auto_find_prompt: Optional[bool] = None,
        strip_prompt: Optional[bool] = None,
        strip_command: Optional[bool] = None,
        normalize: Optional[bool] = None,
        cmd_verify: Optional[bool] = None
    ) -> str:
        """
        Execute a command and return the raw result

        Args:
            command: The command to execute
            expect_string: Optional pattern to search for in the output
            read_timeout: Optional timeout in seconds for reading output
            auto_find_prompt: Optional flag to automatically find prompt
            strip_prompt: Optional flag to strip prompt from output
            strip_command: Optional flag to strip command from output
            normalize: Optional flag to normalize line feeds
            cmd_verify: Optional flag to verify command echoing

        Returns:
            str: The command output as a string

        Raises:
            RuntimeError: If command execution fails or times out
        """
        ...

    def execute_command_with_result(
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
        Execute a command on the device and return a structured result,
        including checking for device-specific error patterns in the output.

        Args:
            command: The command to execute
            expect_string: Optional pattern to search for in the output
            read_timeout: Optional timeout in seconds for reading output
            auto_find_prompt: Optional flag to automatically find prompt
            strip_prompt: Optional flag to strip prompt from output
            strip_command: Optional flag to strip command from output
            normalize: Optional flag to normalize line feeds
            cmd_verify: Optional flag to verify command echoing

        Returns:
            CommandResult: Result containing the command output and status
        """
        ...

class PyCommandResult:
    """
    Result of a single command execution on a network device.

    This class encapsulates the output and status of a command executed on
    a network device. It provides structured access to the command result
    and metadata about the execution.

    Attributes:
        device_id: The hostname or identifier of the device
        device_type: The type of device (e.g., 'cisco_ios', 'juniper_junos')
        command: The command that was executed
        output: The text output returned by the command
        start_time: When the command started executing (ISO format)
        end_time: When the command finished executing (ISO format)
        duration_ms: How long the command took to execute in milliseconds
        status: The execution status ('Success', 'Failed', 'Timeout', 'Skipped')
        error: Error message if the command failed

    Examples:
        ```python
        # Execute a command and check the result
        result = device.send_command("show version")

        if result.is_success():
            print(f"Command output: {result.output}")
        else:
            print(f"Command failed: {result.error}")
        ```
    """

    device_id: str
    """The hostname or identifier of the device where the command was executed"""

    device_type: str
    """The device type (e.g., 'cisco_ios', 'juniper_junos')"""

    command: str
    """The command that was executed"""

    output: Optional[str]
    """The text output from the command, or None if the command failed"""

    start_time: str
    """When the command started executing, in ISO format"""

    end_time: str
    """When the command finished executing, in ISO format"""

    duration_ms: int
    """How long the command took to execute, in milliseconds"""

    status: str
    """The execution status ('Success', 'Failed', 'Timeout', 'Skipped')"""

    error: Optional[str]
    """Error message if the command failed, or None if successful"""

    def to_dict(self) -> Dict[str, Any]:
        """
        Convert the command result to a Python dictionary.

        Returns:
            A dictionary containing all result attributes
        """
        ...

    def __str__(self) -> str:
        """
        Get a string representation of the command result.

        Returns:
            A formatted string showing command, device, status, and duration
        """
        ...

    def is_success(self) -> bool:
        """
        Check if the command was successful.

        Returns:
            True if the command executed successfully, False otherwise
        """
        ...

    def is_failure(self) -> bool:
        """
        Check if the command failed.

        Returns:
            True if the command failed, False otherwise
        """
        ...

    def is_timeout(self) -> bool:
        """
        Check if the command timed out.

        Returns:
            True if the command timed out, False otherwise
        """
        ...

class PyBatchCommandResults:
    """
    Results of executing commands on multiple devices.

    This class provides methods to access and analyze the results of batch command
    execution across multiple devices. It includes utilities for filtering and
    formatting results.

    Attributes:
        command_count: Total number of commands executed
        success_count: Number of successful commands
        failure_count: Number of failed commands
        device_count: Number of devices that commands were executed on
        duration_ms: Total execution time in milliseconds

    Examples:
        ```python
        # Execute commands on multiple devices
        manager = PyParallelExecutionManager()
        results = manager.execute_commands_on_all(
            [device1_config, device2_config],
            ["show version", "show ip interface brief"]
        )

        # Get all successful results
        success_results = results.get_successful_results()
        
        # Get results for a specific device
        device_results = results.get_device_results("router1")
        
        # Export results
        csv_data = results.to_csv()
        json_data = results.to_json()
        ```
    """

    command_count: int
    """Total number of commands executed across all devices"""

    success_count: int
    """Number of commands that completed successfully"""

    failure_count: int
    """Number of commands that failed"""

    device_count: int
    """Number of devices that commands were executed on"""

    duration_ms: int
    """Total execution time in milliseconds"""

    def get_device_results(self, device_id: str) -> Optional[List[PyCommandResult]]:
        """
        Get all results for a specific device.

        Args:
            device_id: The device identifier or hostname

        Returns:
            A list of PyCommandResult objects for the device, or None if not found
        """
        ...

    def get_all_results(self) -> List[PyCommandResult]:
        """
        Get all command results across all devices.

        Returns:
            A list of all PyCommandResult objects
        """
        ...

    def get_successful_results(self) -> List[PyCommandResult]:
        """
        Get all successful command results.

        Returns:
            A list of PyCommandResult objects with 'Success' status
        """
        ...

    def get_failed_results(self) -> List[PyCommandResult]:
        """
        Get all failed command results.

        Returns:
            A list of PyCommandResult objects with 'Failed' status
        """
        ...

    def get_command_results(self, command: str) -> List[PyCommandResult]:
        """
        Get results for a specific command across all devices.

        Args:
            command: The command to filter by

        Returns:
            A list of PyCommandResult objects for the specified command
        """
        ...

    def format_as_table(self) -> str:
        """
        Format the results as a table for display.

        Returns:
            A formatted string containing a table of results
        """
        ...

    def to_json(self) -> str:
        """
        Convert the batch results to JSON.

        Returns:
            A JSON string representation of the results
        """
        ...

    def to_csv(self) -> str:
        """
        Convert the batch results to CSV.

        Returns:
            A CSV string representation of the results
        """
        ...

    def compare_outputs(self, command: str) -> Dict[str, str]:
        """
        Compare command outputs across devices.

        Args:
            command: The command to compare across devices

        Returns:
            A dictionary mapping device IDs to their command outputs
        """
        ...

class PyParallelExecutionConfig:
    """
    Configuration for parallel execution of network commands.

    This class configures how commands are executed in parallel across devices,
    including timeout settings, concurrency limits, and error handling behavior.

    Attributes:
        num_threads: Maximum number of devices to connect to simultaneously
        command_timeout_sec: Maximum time allowed for a single command to complete
        connect_timeout_sec: Maximum time allowed for establishing a connection
        stop_on_error: Whether to stop all execution when any error occurs
        retry_failed_devices: Whether to retry failed devices after all others complete
        max_retries_per_device: Maximum number of retry attempts per device
        delay_between_commands_ms: Delay between executing commands on the same device

    Examples:
        ```python
        # Default configuration
        config = PyParallelExecutionConfig()

        # Custom configuration
        config = PyParallelExecutionConfig(
            num_threads=20,
            command_timeout_sec=60,
            connect_timeout_sec=30,
            stop_on_error=False
        )
        
        # Use with parallel execution manager
        manager = PyParallelExecutionManager(config)
        ```
    """

    def __init__(
        self,
        num_threads: int = 10,
        command_timeout_sec: int = 30,
        connect_timeout_sec: int = 15,
        stop_on_error: bool = False,
        retry_failed_devices: bool = False,
        max_retries_per_device: int = 1,
        delay_between_commands_ms: int = 100
    ) -> None:
        """
        Initialize a new parallel execution configuration.

        Args:
            num_threads: Maximum number of devices to connect to simultaneously
            command_timeout_sec: Maximum time allowed for a single command to complete
            connect_timeout_sec: Maximum time allowed for establishing a connection
            stop_on_error: Whether to stop all execution when any error occurs
            retry_failed_devices: Whether to retry failed devices after all others complete
            max_retries_per_device: Maximum number of retry attempts per device
            delay_between_commands_ms: Delay between executing commands on the same device
        """
        ...

class PyParallelExecutionManager:
    """
    Manager for parallel execution of commands across multiple network devices.

    This class provides methods to execute commands in parallel across multiple
    network devices, with configurable concurrency and error handling behavior.

    Examples:
        ```python
        # Create a manager with default configuration
        manager = PyParallelExecutionManager()
        
        # Create device configurations
        configs = [
            PyDeviceConfig(device_type="cisco_ios", host="router1", username="admin", password="pass"),
            PyDeviceConfig(device_type="cisco_ios", host="router2", username="admin", password="pass")
        ]
        
        # Execute the same commands on all devices
        results = manager.execute_commands_on_all(configs, [
            "show version", 
            "show ip interface brief"
        ])
        
        # Execute device-specific commands
        device_commands = {
            config1: ["show version", "show ip route"],
            config2: ["show version", "show interfaces"]
        }
        results = manager.execute_commands(device_commands)
        
        # Process results
        print(f"Success: {results.success_count}/{results.command_count}")
        print(results.format_as_table())
        ```
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
        Initialize a new parallel execution manager.

        Args:
            max_concurrency: Maximum number of devices to connect to simultaneously
            command_timeout_seconds: Maximum time allowed for a single command to complete (in seconds)
            connection_timeout_seconds: Maximum time allowed for establishing a connection (in seconds, deprecated)
            failure_strategy: How to handle failures ('continue_on_device', 'skip_device', 'abort_batch')
            reuse_connections: Whether to reuse connections for multiple commands
        """
        ...

    def set_max_concurrency(self, max_concurrency: int) -> None:
        """
        Set the maximum concurrency for parallel device connections.
        
        Args:
            max_concurrency: Maximum number of devices to connect to simultaneously
        """
        ...

    def set_command_timeout(self, timeout_seconds: int) -> None:
        """
        Set the command timeout in seconds.
        
        Args:
            timeout_seconds: Maximum time allowed for a command to complete
        """
        ...

    def set_connection_timeout(self, timeout_seconds: int) -> None:
        """
        Set the connection timeout in seconds (deprecated).
        
        Args:
            timeout_seconds: Connection timeout (no longer used)
        """
        ...

    def set_failure_strategy(self, strategy: str) -> None:
        """
        Set the failure handling strategy.
        
        Args:
            strategy: Strategy for handling command failures 
                     ('continue_on_device', 'skip_device', 'abort_batch')
        """
        ...

    def set_reuse_connections(self, reuse: bool) -> None:
        """
        Set whether to reuse connections for multiple commands.
        
        Args:
            reuse: Whether to reuse existing connections
        """
        ...

    def execute_command_on_all(
        self,
        configs: List[PyDeviceConfig],
        command: str
    ) -> PyBatchCommandResults:
        """
        Execute a single command on multiple devices in parallel.

        Args:
            configs: List of device configurations to connect to
            command: The command to execute on each device

        Returns:
            Batch command results containing all execution results

        Raises:
            Exception: If there are issues creating devices or during execution
        """
        ...

    def execute_commands_on_all(
        self, 
        device_configs: List[PyDeviceConfig], 
        commands: List[str]
    ) -> PyBatchCommandResults:
        """
        Execute multiple commands on all devices in parallel.

        Args:
            device_configs: List of device configurations to connect to
            commands: List of commands to execute on each device

        Returns:
            Batch command results containing all execution results

        Raises:
            Exception: If there are issues creating devices or during execution
        """
        ...

    def execute_commands(
        self, 
        device_commands: Dict[PyDeviceConfig, List[str]]
    ) -> PyBatchCommandResults:
        """
        Execute device-specific commands in parallel.

        Each device config maps to a list of commands to execute on that device.

        Args:
            device_commands: Dictionary mapping device configs to command lists

        Returns:
            Batch command results containing all execution results

        Raises:
            Exception: If there are issues creating devices or during execution
        """
        ...
        
    def cleanup(self) -> None:
        """
        Close all open connections.
        
        This method should be called when you're done with the manager to 
        release all resources. If using the context manager, this is called
        automatically.
        """
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

class NetworkError(Exception):
    """
    Base exception for all netssh network-related errors.

    This is the parent class for all exceptions raised by the netssh library
    related to network operations.
    """
    pass

class AuthenticationError(NetworkError):
    """
    Raised when authentication to a device fails.

    This exception is raised when the library cannot authenticate to a device
    using the provided credentials.
    """
    pass

class ConnectionError(NetworkError):
    """
    Raised when a connection to a device cannot be established.

    This exception is raised when the library cannot establish a connection
    to a device due to network issues, incorrect hostname, etc.
    """
    pass

class CommandError(NetworkError):
    """
    Raised when a command execution fails.

    This exception is raised when a command execution on a device fails
    for any reason other than timeout.
    """
    pass

class CommandTimeoutError(CommandError):
    """
    Raised when a command execution times out.

    This exception is raised when a command execution takes longer than
    the configured timeout period.
    """
    pass

class ConfigModeError(NetworkError):
    """
    Raised when entering or exiting configuration mode fails.

    This exception is raised when the library cannot enter or exit
    configuration mode on a device.
    """
    pass

def autodetect_device_type(
    host: str, 
    username: str, 
    password: str, 
    port: int = 22,
    enable_password: Optional[str] = None,
    timeout: int = 10
) -> str:
    """
    Automatically detect the device type from a network device.

    This function attempts to connect to the device and determine its type
    based on the device's response patterns.

    Args:
        host: The hostname or IP address of the device
        username: The username for authentication
        password: The password for authentication
        port: The SSH port to connect to (default 22)
        enable_password: The enable password if required (optional)
        timeout: Connection timeout in seconds (default 10)

    Returns:
        A string indicating the detected device type (e.g., 'cisco_ios', 'juniper_junos')
        
    Raises:
        ConnectionError: If connection to the device fails
        AuthenticationError: If authentication to the device fails
        NetworkError: For other network-related errors

    Examples:
        ```python
        try:
            device_type = autodetect_device_type(
                host="192.168.1.1",
                username="admin",
                password="password"
            )
            print(f"Detected device type: {device_type}")
        except NetworkError as e:
            print(f"Error detecting device type: {e}")
        ```
    """
    ...

def set_log_level(level: str) -> None:
    """
    Set the global logging level for the netssh library.

    Args:
        level: The logging level to set ("error", "warn", "info", "debug", "trace")

    Examples:
        ```python
        # Enable debug logging
        set_log_level("debug")
        
        # Set to minimal logging
        set_log_level("error")
        ```
    """
    ...