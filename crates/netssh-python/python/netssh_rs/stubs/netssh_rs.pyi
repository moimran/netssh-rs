"""
Type stubs for netssh_rs Rust extension module.

This file provides type hints for the Rust-generated Python bindings,
enabling proper IntelliSense support in VSCode and other editors.
"""

from typing import Dict, List, Any, Optional, Union, Tuple, Type, TypeVar, overload
from types import TracebackType

T = TypeVar('T', bound='PyNetworkDevice')

def initialize_logging(debug: bool = False, console: bool = False) -> None:
    """
    Initialize logging for the netssh_rs module.

    Configures the logging system for the netssh-rs library. This should typically
    be called once at the start of your application.

    Args:
        debug: When True, enables detailed debug logging for troubleshooting
        console: When True, logs to console instead of file. Useful for interactive sessions.
              When False, logs to file location specified in configuration.

    Examples:
        ```python
        # Basic logging to file
        initialize_logging()

        # Debug logging to console
        initialize_logging(debug=True, console=True)
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

    def enter_config_mode(self, config_command: Optional[str] = None) -> None:
        """
        Enter configuration mode.

        Switches the device session to configuration mode, where changes to the
        device configuration can be made. For most devices, this is equivalent
        to the "configure terminal" command.

        Args:
            config_command: Optional custom configuration command if the default
                            command for the device type is not appropriate

        Raises:
            RuntimeError: If entering config mode fails (e.g., permission denied)

        Examples:
            ```python
            # Enter config mode with default command
            device.enter_config_mode()

            # Use custom config command
            device.enter_config_mode(config_command="conf t")
            ```
        """
        ...

    def exit_config_mode(self, exit_command: Optional[str] = None) -> None:
        """
        Exit configuration mode.

        Returns from configuration mode to enable/privileged mode. For most
        devices, this is equivalent to the "end" or "exit" command.

        Args:
            exit_command: Optional custom exit command if the default command
                        for the device type is not appropriate

        Raises:
            RuntimeError: If exiting config mode fails

        Examples:
            ```python
            # Exit config mode with default command
            device.exit_config_mode()

            # Use custom exit command
            device.exit_config_mode(exit_command="end")
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
    ) -> str:
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
            The combined output from all configuration commands

        Raises:
            RuntimeError: If configuration fails or device rejects commands

        Examples:
            ```python
            # Basic configuration example
            output = device.send_config_set([
                "interface GigabitEthernet0/1",
                "description WAN Link",
                "ip address 192.168.1.1 255.255.255.0",
                "no shutdown"
            ])

            # Advanced configuration with custom options
            output = device.send_config_set(
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

    def set_terminal_width(self, width: int) -> None:
        """
        Set terminal width.

        Configures the terminal width to prevent line wrapping or truncation
        which can cause parsing issues.

        Args:
            width: Width in characters (typically 511 or larger is recommended)

        Raises:
            RuntimeError: If setting terminal width fails

        Examples:
            ```python
            # Set a wide terminal to prevent wrapping
            device.set_terminal_width(511)
            ```
        """
        ...

    def disable_paging(self) -> None:
        """
        Disable paging on the device.

        Turns off the "more" prompt or paging behavior when output exceeds
        the screen length. This ensures that commands return all output at once
        without requiring user interaction.

        Raises:
            RuntimeError: If disabling paging fails

        Examples:
            ```python
            # Disable paging for long outputs
            device.disable_paging()

            # Now commands like "show run" will return complete output
            full_config = device.send_command("show running-config")
            ```
        """
        ...

    def set_base_prompt(self) -> str:
        """
        Set and return the base prompt.

        Determines the device's command prompt pattern for accurate command
        completion detection. This is typically called automatically during
        connection setup.

        Returns:
            The base prompt string detected from the device

        Raises:
            RuntimeError: If setting base prompt fails

        Examples:
            ```python
            # Manually update the base prompt if needed
            prompt = device.set_base_prompt()
            print(f"Detected prompt: {prompt}")
            ```
        """
        ...

    def save_configuration(self) -> None:
        """
        Save or commit the configuration.

        Persists configuration changes using the appropriate command for the
        device type (e.g., "write memory" for Cisco IOS, "commit" for Juniper).

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
            device.save_configuration()
            ```
        """
        ...

    def send_command(self, command: str) -> str:
        """
        Send a command to the device and return the output.

        Executes a single command on the device and waits for completion.
        This method handles command termination detection and timing.

        Args:
            command: The command string to execute on the device

        Returns:
            The command output as a string, with prompt removed

        Raises:
            RuntimeError: If command execution fails or times out

        Examples:
            ```python
            # Get interface status
            output = device.send_command("show ip interface brief")

            # Check version information
            version = device.send_command("show version")
            ```
        """
        ...

    def send_commands(self, commands: List[str]) -> List[str]:
        """
        Send multiple commands to the device and return the outputs.

        Executes a list of commands sequentially and collects their outputs.
        This is more efficient than making multiple send_command calls.

        Args:
            commands: List of command strings to execute

        Returns:
            List of command outputs as strings, in the same order as commands

        Raises:
            RuntimeError: If any command execution fails

        Examples:
            ```python
            # Get multiple show commands at once
            results = device.send_commands([
                "show version",
                "show ip interface brief",
                "show vlan brief"
            ])

            # Access results by index
            version_output = results[0]
            interface_output = results[1]
            ```
        """
        ...

    def get_device_info(self) -> "PyDeviceInfo":
        """
        Get detailed information about the device.

        Returns:
            A PyDeviceInfo object containing device details

        Raises:
            RuntimeError: If getting device info fails
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

class PyCommandResult:
    """
    Result of a single command execution on a network device.

    This class encapsulates the output and status of a command executed on
    a network device. It provides structured access to the command result
    and metadata about the execution.

    Attributes:
        command: The command that was executed
        output: The raw text output from the command
        success: Whether the command completed successfully
        hostname: The hostname of the device on which the command was executed
        device_type: The type of the device on which the command was executed

    Examples:
        ```python
        # Execute a command and check the result
        result = device.send_command("show version")

        if result.success:
            print(f"Command output: {result.output}")
        else:
            print(f"Command failed on {result.hostname}")

        # Access structured data if supported (not all commands/devices)
        structured_data = result.get_structured_data()
        if structured_data:
            print(f"Parsed data: {structured_data}")
        ```
    """

    command: str
    """The command string that was executed on the device"""

    output: str
    """The raw text output returned by the device"""

    success: bool
    """Whether the command completed successfully (True) or failed (False)"""

    hostname: str
    """The hostname of the device where the command was executed"""

    device_type: str
    """The device type (e.g., 'cisco_ios', 'juniper_junos') of the device"""

    def __init__(
        self,
        command: str,
        output: str,
        success: bool,
        hostname: str,
        device_type: str
    ) -> None:
        """
        Initialize a new command result.

        Creates a new command result with the specified parameters. This is typically
        not called directly, but is returned by PyNetworkDevice.send_command().

        Args:
            command: The command string that was executed
            output: The raw text output from the command
            success: Whether the command completed successfully
            hostname: The hostname of the device
            device_type: The type of the device
        """
        ...

    def get_structured_data(self) -> Optional[Dict[str, Any]]:
        """
        Get structured data parsed from the command output.

        For supported command types and devices, this method parses the raw
        text output into a structured dictionary. This uses TextFSM templates
        when available for the specific command and device type.

        Returns:
            A dictionary of parsed data if available, None otherwise

        Examples:
            ```python
            # Execute a command and get structured data
            result = device.send_command("show ip interface brief")
            data = result.get_structured_data()

            if data:
                # Process structured data
                for interface in data:
                    print(f"Interface {interface['interface']} has IP {interface['ip_address']}")
            else:
                # Fall back to raw text processing
                print(f"Raw output: {result.output}")
            ```
        """
        ...

    def __str__(self) -> str:
        """
        Get string representation of the command result.

        Returns:
            A string representation of the command result including the command
            and a preview of the output
        """
        ...

class PyBatchCommandResults:
    """
    Collection of command results from a batch execution.

    This class encapsulates results from executing multiple commands, either
    on a single device or across multiple devices in parallel. It provides
    methods to access and analyze the results.

    Attributes:
        success: Whether all commands completed successfully
        results: List of individual PyCommandResult objects

    Examples:
        ```python
        # Execute multiple commands on multiple devices
        configs = [config1, config2, config3]  # List of PyDeviceConfig objects
        commands = ["show version", "show ip interface brief"]

        manager = PyParallelExecutionManager()
        batch_results = manager.execute_commands_on_all(configs, commands)

        # Check overall success
        if batch_results.success:
            print("All commands completed successfully")

        # Access individual results
        for result in batch_results.results:
            print(f"Command '{result.command}' on {result.hostname}: {'Success' if result.success else 'Failed'}")

        # Get results for a specific device
        device_results = batch_results.get_results_for_device("router1")
        for result in device_results:
            print(f"Output from {result.command}: {result.output}")
        ```
    """

    success: bool
    """Whether all commands in the batch completed successfully"""

    results: List[PyCommandResult]
    """List of individual command results"""

    def __init__(self, results: List[PyCommandResult]) -> None:
        """
        Initialize a new batch command results.

        Creates a new batch command results object with the specified parameters.
        This is typically not called directly, but is returned by PyParallelExecutionManager
        methods.

        Args:
            results: List of PyCommandResult objects
        """
        ...

    def get_results_for_device(self, hostname: str) -> List[PyCommandResult]:
        """
        Get all command results for a specific device.

        Filters the results to only include those from the specified hostname.

        Args:
            hostname: The hostname of the device to filter by

        Returns:
            A list of PyCommandResult objects for the specified device

        Examples:
            ```python
            # Get results for a specific device
            device_results = batch_results.get_results_for_device("router1")

            # Process device-specific results
            for result in device_results:
                print(f"Command: {result.command}")
                print(f"Output: {result.output}")
            ```
        """
        ...

    def get_results_for_command(self, command: str) -> List[PyCommandResult]:
        """
        Get all results for a specific command.

        Filters the results to only include those for the specified command,
        potentially across multiple devices.

        Args:
            command: The command string to filter by

        Returns:
            A list of PyCommandResult objects for the specified command

        Examples:
            ```python
            # Get results for a specific command across all devices
            version_results = batch_results.get_results_for_command("show version")

            # Process command-specific results
            for result in version_results:
                print(f"Device: {result.hostname}")
                print(f"Output: {result.output}")
            ```
        """
        ...

    def get_result(self, hostname: str, command: str) -> Optional[PyCommandResult]:
        """
        Get a specific result by hostname and command.

        Finds the result for a specific command on a specific device.

        Args:
            hostname: The hostname of the device
            command: The command string

        Returns:
            The PyCommandResult for the specified device and command, or None if not found

        Examples:
            ```python
            # Get a specific command result for a specific device
            result = batch_results.get_result("router1", "show version")

            if result:
                print(f"Version info for {result.hostname}: {result.output}")
            else:
                print("Result not found")
            ```
        """
        ...

    def __str__(self) -> str:
        """
        Get string representation of the batch results.

        Returns:
            A string summary of the batch results including success rate and command count
        """
        ...

class PyParallelExecutionManager:
    """
    Manager for executing commands on multiple devices in parallel.

    This class provides functionality to execute commands on multiple network
    devices simultaneously, enabling efficient command execution across a network.
    It handles connection pooling, command dispatching, and result aggregation.

    The PyParallelExecutionManager uses a thread pool to execute commands in
    parallel, with configurable concurrency limits and timeout settings.

    Examples:
        ```python
        # Create device configurations
        device1 = PyDeviceConfig(
            device_type="cisco_ios",
            host="192.168.1.1",
            username="admin",
            password="cisco123"
        )

        device2 = PyDeviceConfig(
            device_type="juniper_junos",
            host="192.168.1.2",
            username="admin",
            password="juniper123"
        )

        # Create a parallel execution manager
        manager = PyParallelExecutionManager()

        # Set execution parameters (optional)
        manager.set_max_concurrency(10)
        manager.set_command_timeout(30)

        # Execute commands on both devices in parallel
        commands = ["show version", "show ip interface brief"]
        results = manager.execute_commands_on_all([device1, device2], commands)

        # Process results
        for result in results.results:
            print(f"Device: {result.hostname}, Command: {result.command}")
            print(f"Success: {result.success}")
            print(f"Output: {result.output}")
        ```
    """

    def __init__(self) -> None:
        """
        Initialize a new parallel execution manager.

        Creates a new parallel execution manager with default settings. The default
        values can be customized using the setter methods.

        Examples:
            ```python
            # Create with default settings
            manager = PyParallelExecutionManager()

            # Create and customize
            manager = PyParallelExecutionManager()
            manager.set_max_concurrency(20)
            manager.set_command_timeout(60)
            ```
        """
        ...

    def set_max_concurrency(self, max_workers: int) -> None:
        """
        Set the maximum number of parallel workers.

        Configures the size of the thread pool used for parallel execution.
        Higher values allow more devices to be accessed simultaneously, but
        consume more system resources.

        Args:
            max_workers: Maximum number of parallel worker threads

        Examples:
            ```python
            # Set to 10 workers (execute on 10 devices simultaneously)
            manager.set_max_concurrency(10)

            # For larger networks, increase accordingly
            manager.set_max_concurrency(50)
            ```
        """
        ...

    def set_command_timeout(self, timeout_seconds: int) -> None:
        """
        Set the command execution timeout.

        Configures the maximum time allowed for a single command to complete
        before it's considered failed. This is distinct from the connection
        timeout in the PyDeviceConfig.

        Args:
            timeout_seconds: Maximum time in seconds for a command to complete

        Examples:
            ```python
            # Set a 30 second timeout for all commands
            manager.set_command_timeout(30)

            # For commands that might take longer (e.g., transfers)
            manager.set_command_timeout(300)  # 5 minutes
            ```
        """
        ...

    def set_connection_timeout(self, timeout_seconds: int) -> None:
        """
        Set the connection establishment timeout.

        Configures the maximum time allowed to establish a connection to a device
        before the connection attempt is considered failed.

        Args:
            timeout_seconds: Maximum time in seconds to establish a connection

        Examples:
            ```python
            # Set a 10 second connection timeout
            manager.set_connection_timeout(10)

            # For slow or remote networks, increase the timeout
            manager.set_connection_timeout(30)
            ```
        """
        ...

    def set_failure_strategy(self, strategy: str) -> None:
        """
        Set the failure handling strategy.

        Configures how the manager responds when a device connection or command
        execution fails. Available strategies include "continue" (proceed with
        other devices) and "abort" (stop all operations).

        Args:
            strategy: The failure strategy to use ("continue" or "abort")

        Examples:
            ```python
            # Continue with other devices even if some fail
            manager.set_failure_strategy("continue")

            # Abort all operations if any device fails
            manager.set_failure_strategy("abort")
            ```
        """
        ...

    def set_reuse_connections(self, reuse: bool) -> None:
        """
        Set whether to reuse connections between command executions.

        When enabled, connections to devices are kept open between command
        executions, improving performance for multiple commands. When disabled,
        new connections are established for each execution.

        Args:
            reuse: True to reuse connections, False to establish new ones each time

        Examples:
            ```python
            # Enable connection reuse for better performance
            manager.set_reuse_connections(True)

            # Disable for more isolation between commands
            manager.set_reuse_connections(False)
            ```
        """
        ...

    def execute_command_on_all(self, configs: List[PyDeviceConfig], command: str) -> PyBatchCommandResults:
        """
        Execute a single command on multiple devices.

        Connects to all specified devices in parallel and executes the same
        command on each. This is useful for commands like "show version" that
        you want to run on many devices.

        Args:
            configs: List of device configurations to connect to
            command: The command string to execute on all devices

        Returns:
            A PyBatchCommandResults object containing all command results

        Raises:
            RuntimeError: If the execution fails catastrophically

        Examples:
            ```python
            # Execute the same command on multiple devices
            results = manager.execute_command_on_all(
                [device1, device2, device3],
                "show version"
            )

            # Check overall success
            if results.success:
                print("Command executed successfully on all devices")

            # Process individual results
            for result in results.results:
                print(f"Device: {result.hostname}")
                print(f"Output: {result.output}")
            ```
        """
        ...

    def execute_commands(self, config: PyDeviceConfig, commands: List[str]) -> PyBatchCommandResults:
        """
        Execute multiple commands on a single device.

        Connects to the specified device and executes a sequence of commands.
        This is useful for gathering multiple pieces of information from a
        single device.

        Args:
            config: The device configuration to connect to
            commands: List of command strings to execute sequentially

        Returns:
            A PyBatchCommandResults object containing all command results

        Raises:
            RuntimeError: If the execution fails catastrophically

        Examples:
            ```python
            # Execute multiple commands on one device
            results = manager.execute_commands(
                device_config,
                ["show version", "show ip interface brief", "show vlan"]
            )

            # Process results by command
            for command in ["show version", "show ip interface brief", "show vlan"]:
                cmd_results = results.get_results_for_command(command)
                if cmd_results:
                    print(f"Output of {command}: {cmd_results[0].output}")
            ```
        """
        ...

    def execute_commands_on_all(self, configs: List[PyDeviceConfig], commands: List[str]) -> PyBatchCommandResults:
        """
        Execute multiple commands on multiple devices.

        Connects to all specified devices in parallel and executes the same
        sequence of commands on each. This is the most comprehensive method,
        allowing you to run a command set on many devices simultaneously.

        Args:
            configs: List of device configurations to connect to
            commands: List of command strings to execute on each device

        Returns:
            A PyBatchCommandResults object containing all command results

        Raises:
            RuntimeError: If the execution fails catastrophically

        Examples:
            ```python
            # Execute multiple commands on multiple devices
            device_configs = [device1_config, device2_config, device3_config]
            commands = ["show version", "show ip interface brief"]

            results = manager.execute_commands_on_all(device_configs, commands)

            # Process all results
            for result in results.results:
                print(f"Device: {result.hostname}")
                print(f"Command: {result.command}")
                print(f"Output: {result.output}")

            # Get results for a specific device and command
            router1_version = results.get_result("router1", "show version")
            if router1_version:
                print(f"Router1 version: {router1_version.output}")
            ```
        """
        ...

    def cleanup(self) -> None:
        """
        Clean up all resources used by the manager.

        Closes all open connections and releases thread pool resources.
        This should be called when the manager is no longer needed to
        prevent resource leaks.

        Examples:
            ```python
            # Execute commands
            results = manager.execute_commands_on_all(devices, commands)

            # Process results...

            # Clean up when done
            manager.cleanup()
            ```
        """
        ...