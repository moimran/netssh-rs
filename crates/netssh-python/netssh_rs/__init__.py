"""
Python bindings for the netssh-rs Rust library.

This module provides network device connectivity and automation capabilities.
"""

# Import everything from the Rust module (will be available when the package is built)
try:
    # Import directly from the .so file
    from netssh_rs.netssh_rs import *
    
    # Re-export all symbols for better IDE support
    __all__ = [
        'initialize_logging',
        'PyDeviceConfig',
        'PyDeviceInfo',
        'PyNetworkDevice',
        'PyCommandResult',
        'PyBatchCommandResults',
        'PyParallelExecutionManager'
    ]
except ImportError:
    # This will happen during development or when type checking
    # We'll provide dummy implementations for IDE support
    import sys
    import os
    import typing
    
    # Add a hint for IDE to find our stub files
    if typing.TYPE_CHECKING:
        # These imports are only used for type checking
        from .stubs.netssh_rs import (
            initialize_logging,
            PyDeviceConfig,
            PyDeviceInfo,
            PyNetworkDevice,
            PyCommandResult,
            PyBatchCommandResults,
            PyParallelExecutionManager
        )
        
        __all__ = [
            'initialize_logging',
            'PyDeviceConfig',
            'PyDeviceInfo',
            'PyNetworkDevice',
            'PyCommandResult',
            'PyBatchCommandResults',
            'PyParallelExecutionManager'
        ]