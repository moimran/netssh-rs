from typing import Dict, List, Optional, Union, Any

class PyDeviceConfig:
    """Configuration for a network device connection."""
    
    def __init__(
        self,
        device_type: str,
        host: str,
        username: str,
        password: str,
        port: int = 22,
        secret: Optional[str] = None
    ) -> None:
        """
        Initialize a new device configuration.
        
        Args:
            device_type: The type of device (e.g., 'cisco_ios', 'juniper')
            host: The hostname or IP address of the device
            username: The username for authentication
            password: The password for authentication
            port: The SSH port (default: 22)
            secret: The enable secret for privileged mode (if required)
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

class PyNetworkDevice:
    """Network device connection handler."""
    
    @staticmethod
    def create(config: PyDeviceConfig) -> 'PyNetworkDevice':
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
        Connect to the network device.
        
        Raises:
            ConnectionError: If connection fails
        """
        ...
    
    def close(self) -> None:
        """Close the connection to the device."""
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