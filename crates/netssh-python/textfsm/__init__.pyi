"""
Type stubs for textfsm module.

This file provides type hints for the textfsm module to enable proper
IntelliSense support in Python editors.
"""

from typing import Dict, List, Any, Optional, Union, Tuple, TextIO

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

class TextFSM:
    """Template based state machine for parsing semi-formatted text."""
    
    def __init__(self, template: TextIO) -> None:
        """
        Initialize TextFSM with a template file.
        
        Args:
            template: Template file object
        """
        ...
    
    def ParseText(self, text: str) -> List[List[str]]:
        """
        Parse text using the loaded template.
        
        Args:
            text: Text to parse
        
        Returns:
            List of records, each a list of string values
        """
        ...
    
    @property
    def header(self) -> List[str]:
        """
        Get the field names from the template.
        
        Returns:
            List of field names
        """
        ... 