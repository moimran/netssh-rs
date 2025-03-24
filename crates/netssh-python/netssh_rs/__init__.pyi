"""
Type stubs for netssh_rs module.

This file provides type hints for the netssh_rs module to enable proper
IntelliSense support in Python editors.
"""

from typing import Dict, List, Any, Optional, Union
from types import TracebackType

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
        """Initialize device configuration."""
        ...

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
    def create(cls, config: PyDeviceConfig) -> "PyNetworkDevice":
        """Create a new network device connection."""
        ...
    
    def connect(self) -> None:
        """Connect to the device."""
        ...
    
    def close(self) -> None:
        """Close the connection to the device."""
        ...
    
    def check_config_mode(self) -> bool:
        """Check if the device is in configuration mode."""
        ...
    
    def enter_config_mode(self, config_command: Optional[str] = None) -> None:
        """Enter configuration mode."""
        ...
    
    def exit_config_mode(self, exit_command: Optional[str] = None) -> None:
        """Exit configuration mode."""
        ...
    
    def session_preparation(self) -> None:
        """Prepare the session after connection."""
        ...
    
    def terminal_settings(self) -> None:
        """Configure terminal settings."""
        ...
    
    def set_terminal_width(self, width: int) -> None:
        """Set terminal width."""
        ...
    
    def disable_paging(self) -> None:
        """Disable paging on the device."""
        ...
    
    def set_base_prompt(self) -> str:
        """Set and return the base prompt."""
        ...
    
    def save_configuration(self) -> None:
        """Save or commit the configuration."""
        ...
    
    def send_command(self, command: str) -> str:
        """Send a command to the device and return the output."""
        ...
    
    def get_device_type(self) -> str:
        """Get the device type (vendor/model)."""
        ...
    
    def __enter__(self) -> "PyNetworkDevice":
        """Context manager entry."""
        ...
    
    def __exit__(
        self,
        exc_type: Optional[type],
        exc_val: Optional[Exception],
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
        """Convert the command result to a dictionary."""
        ...

class PyBatchCommandResults:
    """Results of batch command execution."""
    
    def get_device_results(self, device_id: str) -> Optional[List[PyCommandResult]]:
        """Get all results for a specific device."""
        ...
    
    def get_all_results(self) -> List[PyCommandResult]:
        """Get all command results."""
        ...
    
    def get_successful_results(self) -> List[PyCommandResult]:
        """Get all successful command results."""
        ...
    
    def get_failed_results(self) -> List[PyCommandResult]:
        """Get all failed command results."""
        ...
    
    def get_command_results(self, command: str) -> List[PyCommandResult]:
        """Get results for a specific command."""
        ...
    
    def format_as_table(self) -> str:
        """Format results as an ASCII table."""
        ...
    
    def to_json(self) -> str:
        """Convert results to JSON format."""
        ...
    
    def to_csv(self) -> str:
        """Convert results to CSV format."""
        ...
    
    def compare_outputs(self, command: str) -> Dict[str, Dict[str, List[str]]]:
        """Compare outputs for the same command across devices."""
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
        """Initialize a parallel execution manager."""
        ...
    
    def set_max_concurrency(self, max_concurrency: int) -> None:
        """Set maximum number of concurrent connections."""
        ...
    
    def set_command_timeout(self, timeout_seconds: int) -> None:
        """Set command timeout."""
        ...
    
    def set_connection_timeout(self, timeout_seconds: int) -> None:
        """Set connection timeout."""
        ...
    
    def set_failure_strategy(self, strategy: str) -> None:
        """Set failure strategy."""
        ...
    
    def set_reuse_connections(self, reuse: bool) -> None:
        """Set whether to reuse connections."""
        ...
    
    def execute_command_on_all(self, devices: List[PyDeviceConfig], command: str) -> PyBatchCommandResults:
        """Execute a command on all devices."""
        ...
    
    def execute_commands_on_all(self, devices: List[PyDeviceConfig], commands: List[str]) -> PyBatchCommandResults:
        """Execute multiple commands on all devices."""
        ...
    
    def execute_commands(self, device_commands: Dict[PyDeviceConfig, Union[str, List[str]]]) -> PyBatchCommandResults:
        """Execute specific commands on specific devices."""
        ...
    
    def cleanup(self) -> None:
        """Close all open connections."""
        ...
    
    def __enter__(self) -> "PyParallelExecutionManager":
        """Context manager entry."""
        ...
    
    def __exit__(
        self,
        exc_type: Optional[type],
        exc_val: Optional[Exception],
        exc_tb: Optional[TracebackType]
    ) -> bool:
        """Context manager exit."""
        ... 