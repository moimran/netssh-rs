"""
Python bindings for the netssh-rs Rust library.

This module provides network device connectivity and automation capabilities.
"""

# Import everything from the Rust module (will be available when the package is built)
try:
    # Import directly from the .so file
    from netssh_rs.netssh_rs import *
except ImportError:
    # This will happen during development or when type checking
    pass 