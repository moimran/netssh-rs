"""
Python bindings for netssh-rs - SSH connection handler for network devices.

This module provides Python bindings to the netssh-rs Rust library, which implements
SSH connection handling and command execution for network devices.

Main features:
- Fast, concurrent SSH connections to network devices
- TextFSM parsing of command outputs
- Support for various network device types
- Parallel command execution
- Error handling and recovery
"""

# Re-export the Rust module
import importlib
import sys
import os

# Add the current directory to path to ensure netssh_rs can be imported
sys.path.insert(0, os.path.dirname(__file__))

# Import the parsing helper module
from netssh_rs_parse import (
    parse_command_output,
    get_available_templates,
    get_platforms,
    get_supported_commands
)

# Try to import the Rust bindings
try:
    from netssh_rs import *
except ImportError:
    # Handle the case where the Rust library isn't built yet
    print("Error: netssh_rs Rust bindings not found. Please build the package first.")

# Export the parse helper functions
__all__ = [
    'parse_command_output',
    'get_available_templates',
    'get_platforms',
    'get_supported_commands'
] 