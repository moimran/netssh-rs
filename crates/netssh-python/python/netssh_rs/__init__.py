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
        PyParseOptions,
        initialize_logging,
        set_default_session_logging
    )

    # Provide clean aliases without the Py prefix
    DeviceConfig = PyDeviceConfig
    NetworkDevice = PyNetworkDevice
    ParallelExecutionManager = PyParallelExecutionManager
    CommandResult = PyCommandResult
    BatchCommandResults = PyBatchCommandResults
    ParseOptions = PyParseOptions
    
except ImportError as e:
    raise ImportError(f"Error importing netssh_rs Rust module: {e}. Make sure the library is properly installed.") from e

# Export all available functionality
__all__ = [
    # Export only the clean non-Py-prefixed versions
    "DeviceConfig",
    "NetworkDevice",
    "ParallelExecutionManager",
    "CommandResult",
    "BatchCommandResults",
    "ParseOptions",
    "initialize_logging",
    "set_default_session_logging"
]