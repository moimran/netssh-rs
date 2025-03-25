from typing import Dict, List, Optional, Union, Any

def parse_command_output(
    platform: str,
    command: str,
    output: str,
    template_dir: Optional[str] = None
) -> List[Dict[str, str]]:
    """
    Parse command output using TextFSM templates.
    
    Args:
        platform: The device platform (e.g., 'cisco_ios')
        command: The command that was executed
        output: The command output to parse
        template_dir: Optional directory containing TextFSM templates
        
    Returns:
        List of dictionaries containing the parsed data
        
    Raises:
        ValueError: If no template is found for the platform/command
        ParseError: If parsing fails
    """
    ...

def get_available_templates(template_dir: Optional[str] = None) -> Dict[str, List[str]]:
    """
    Get available TextFSM templates.
    
    Args:
        template_dir: Optional directory containing TextFSM templates
        
    Returns:
        Dictionary mapping platforms to lists of supported commands
    """
    ...

def get_platforms(template_dir: Optional[str] = None) -> List[str]:
    """
    Get list of supported platforms.
    
    Args:
        template_dir: Optional directory containing TextFSM templates
        
    Returns:
        List of platform names
    """
    ...

def get_supported_commands(platform: str, template_dir: Optional[str] = None) -> List[str]:
    """
    Get list of supported commands for a platform.
    
    Args:
        platform: The device platform (e.g., 'cisco_ios')
        template_dir: Optional directory containing TextFSM templates
        
    Returns:
        List of supported command names
        
    Raises:
        ValueError: If the platform is not supported
    """
    ...