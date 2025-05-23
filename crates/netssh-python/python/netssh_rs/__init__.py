"""
netssh-rs: A network automation library for secure shell connections

This module provides a Python interface to the netssh-rs Rust library,
enabling efficient and secure network device automation.
"""

import os
import sys
import logging
from typing import Dict, List, Optional, Union, Any

try:
    # Import the Rust backend with Py-prefixed names
    from netssh_rs.netssh_rs import (
        PyDeviceConfig,
        PyNetworkDevice,
        PyParallelExecutionManager,
        PyCommandResult,
        PyBatchCommandResults,
        initialize_logging
    )
    
    # Provide clean aliases without the Py prefix
    DeviceConfig = PyDeviceConfig
    NetworkDevice = PyNetworkDevice
    ParallelExecutionManager = PyParallelExecutionManager
    CommandResult = PyCommandResult
    BatchCommandResults = PyBatchCommandResults
    
except ImportError as e:
    raise ImportError(f"Error importing netssh_rs Rust module: {e}. Make sure the library is properly installed.") from e

# Import TextFSM parser functions from our local implementation
try:
    # Import from our integrated textfsm module
    from .textfsm import parse_output, parse_output_to_json, NetworkOutputParser
    
    # Make TextFSM utilities available directly from the netssh_rs package
    __all__ = [
        # Export only the clean non-Py-prefixed versions
        "DeviceConfig",
        "NetworkDevice",
        "ParallelExecutionManager",
        "CommandResult",
        "BatchCommandResults",
        "initialize_logging",
        # TextFSM exports
        "parse_output",
        "parse_output_to_json",
        "NetworkOutputParser"
    ]
except ImportError as e:
    logging.warning(f"TextFSM module import error: {e}. TextFSM parsing functions will not be available.")
    # If TextFSM is not available, only expose the core functionality
    __all__ = [
        # Export only the clean non-Py-prefixed versions
        "DeviceConfig",
        "NetworkDevice",
        "ParallelExecutionManager",
        "CommandResult",
        "BatchCommandResults",
        "initialize_logging"
    ]