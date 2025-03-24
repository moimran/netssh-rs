"""
Netssh-rs TextFSM parsing helper module.

This module provides helper functions for working with TextFSM parsing in the netssh-rs library.
"""

import os
import sys
from typing import List, Dict, Any, Optional, Union

# Ensure the textfsm module is in the path
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# Import the TextFSM parsing module
from textfsm.parse import parse_output, ParsingException

def parse_command_output(platform: str, command: str, output: str) -> List[Dict[str, str]]:
    """
    Parse command output using TextFSM templates.
    
    Args:
        platform: The platform/device type (e.g., 'cisco_ios', 'arista_eos')
        command: The command that was executed
        output: The raw command output to parse
        
    Returns:
        List of dictionaries containing parsed data
        
    Raises:
        ParsingException: If parsing fails
    """
    try:
        return parse_output(platform=platform, command=command, data=output)
    except Exception as e:
        raise ParsingException(f"Failed to parse output for command '{command}' on platform '{platform}': {str(e)}")

def get_available_templates() -> List[str]:
    """
    Get a list of available TextFSM templates.
    
    Returns:
        List of template names available in the templates directory
    """
    template_dir = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), 
                               "textfsm", "templates")
    
    templates = []
    if os.path.exists(template_dir):
        for filename in os.listdir(template_dir):
            if filename.endswith(".textfsm"):
                templates.append(filename)
    
    return templates

def get_platforms() -> List[str]:
    """
    Get a list of supported platforms based on available templates.
    
    Returns:
        List of unique platform names supported by templates
    """
    template_dir = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), 
                               "textfsm", "templates")
    
    index_file = os.path.join(template_dir, "index")
    platforms = set()
    
    if os.path.exists(index_file):
        with open(index_file, 'r') as f:
            for line in f:
                if line.startswith('#') or not line.strip():
                    continue
                    
                parts = line.strip().split(',')
                if len(parts) >= 3:
                    platform = parts[2].strip()
                    platforms.add(platform)
    
    return sorted(list(platforms))

def get_supported_commands(platform: str) -> List[str]:
    """
    Get a list of commands supported by TextFSM templates for a specific platform.
    
    Args:
        platform: The platform/device type (e.g., 'cisco_ios', 'arista_eos')
        
    Returns:
        List of commands supported for the platform
    """
    template_dir = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), 
                               "textfsm", "templates")
    
    index_file = os.path.join(template_dir, "index")
    commands = []
    
    if os.path.exists(index_file):
        with open(index_file, 'r') as f:
            for line in f:
                if line.startswith('#') or not line.strip():
                    continue
                    
                parts = line.strip().split(',')
                if len(parts) >= 4:
                    template_platform = parts[2].strip()
                    if template_platform == platform:
                        command = parts[3].strip()
                        commands.append(command)
    
    return commands 