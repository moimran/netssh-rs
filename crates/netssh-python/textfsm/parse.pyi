"""
Type stubs for textfsm.parse module.

This file provides type hints for the TextFSM parser used by the textfsm module.
"""

from typing import Dict, List, Any, Optional, Union, TextIO, Iterator

class TextFSMError(Exception):
    """Base error for TextFSM module."""
    pass

class TextFSMTemplateError(TextFSMError):
    """Error in the TextFSM template."""
    pass

class TextFSMParseError(TextFSMError):
    """Error during parsing."""
    pass

class TextFSM:
    """Template based state machine for parsing semi-formatted text."""
    
    def __init__(self, template: TextIO) -> None:
        """
        Initialize TextFSM with a template file.
        
        Args:
            template: Template file object
        
        Raises:
            TextFSMTemplateError: If template syntax is invalid
        """
        ...
    
    def ParseText(self, text: str) -> List[List[str]]:
        """
        Parse text using the template.
        
        Args:
            text: Text to parse
        
        Returns:
            List of records, each a list of string values
        
        Raises:
            TextFSMParseError: If error during parsing
        """
        ...
    
    def ParseTextToDicts(self, text: str) -> List[Dict[str, str]]:
        """
        Parse text using the template and return a list of dictionaries.
        
        Args:
            text: Text to parse
        
        Returns:
            List of records, each a dictionary of field name to value
            
        Raises:
            TextFSMParseError: If error during parsing
        """
        ...
    
    def Reset(self) -> None:
        """
        Reset the state machine.
        
        Used to initialize state before parsing or to reset after parsing
        if reusing the same TextFSM instance.
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
    
    @property
    def values(self) -> List[List[str]]:
        """
        Get the parsed values.
        
        Returns:
            List of records, each a list of string values
        """
        ...
        
def CleanValue(value: str) -> str:
    """
    Remove special characters that could interfere with TextFSM processing.
    
    Args:
        value: String to clean
    
    Returns:
        Cleaned string
    """
    ...

def CopyableRegexp(regexp_str: str) -> str:
    """
    Make a regexp string that can be used verbatim.
    
    Args:
        regexp_str: Regular expression string
    
    Returns:
        String that can be used as a regular expression verbatim
    """
    ...

def ConvertCommand(cmd: str) -> str:
    """
    Convert command string into a form suitable for matching in templates.
    
    Args:
        cmd: Command string
    
    Returns:
        Converted string suitable for matching
    """
    ... 