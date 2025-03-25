"""
netssh-rs: A network automation library for secure shell connections

This module provides a Python interface to the netssh-rs Rust library,
enabling efficient and secure network device automation.
"""

try:
    from netssh_rs.netssh_rs import (
        PyDeviceConfig,
        PyNetworkDevice,
        PyParallelExecutionManager,
        PyCommandResult,
        PyBatchCommandResults,
        PyDeviceInfo,
        initialize_logging
    )
except ImportError as e:
    raise ImportError(f"Error importing netssh_rs Rust module: {e}. Make sure the library is properly installed.") from e

__all__ = [
    "PyDeviceConfig",
    "PyNetworkDevice",
    "PyParallelExecutionManager",
    "PyCommandResult",
    "PyBatchCommandResults",
    "PyDeviceInfo",
    "initialize_logging"
]