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
from typing import Dict, List, Optional, Union, Any

# Add the current directory to path to ensure netssh_rs can be imported
sys.path.insert(0, os.path.dirname(__file__))

# Import the parsing helper module
from netssh_rs_parse import (
    parse_command_output,
    get_available_templates,
    get_platforms,
    get_supported_commands
)

# Import and re-export the Rust bindings
from netssh_rs import (
    PyDeviceConfig, 
    PyNetworkDevice,
    PyDeviceInfo,
    PyCommandResult,
    PyBatchCommandResults,
    PyParallelExecutionManager,
    initialize_logging
)

# Export the parse helper functions and Rust bindings
__all__ = [
    'PyDeviceConfig',
    'PyNetworkDevice',
    'PyDeviceInfo',
    'PyCommandResult',
    'PyBatchCommandResults',
    'PyParallelExecutionManager',
    'initialize_logging',
    'parse_command_output',
    'get_available_templates',
    'get_platforms',
    'get_supported_commands'
] 