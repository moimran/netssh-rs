#!/usr/bin/env python3
"""
Unit tests for netssh_rs Python bindings.
"""

import unittest
import os
from unittest.mock import patch

# Import the module (will fail if not installed)
try:
    import netssh_rs
except ImportError:
    print("netssh_rs module not found. Run 'maturin develop' to build and install.")
    raise


class TestDeviceConfig(unittest.TestCase):
    """Test the PyDeviceConfig class."""

    def test_create_config(self):
        """Test creating a device configuration."""
        config = netssh_rs.PyDeviceConfig(
            device_type="cisco_ios",
            host="192.168.1.1",
            username="admin",
            password="password",
            port=22,
            timeout_seconds=60,
            secret="enable_secret",
            session_log="logs/test_session.log"
        )
        
        self.assertEqual(config.device_type, "cisco_ios")
        self.assertEqual(config.host, "192.168.1.1")
        self.assertEqual(config.username, "admin")
        self.assertEqual(config.password, "password")
        self.assertEqual(config.port, 22)
        self.assertEqual(config.timeout_seconds, 60)
        self.assertEqual(config.secret, "enable_secret")
        self.assertEqual(config.session_log, "logs/test_session.log")


# Mock tests for NetworkDevice - these don't actually connect to devices
class TestNetworkDevice(unittest.TestCase):
    """Test the PyNetworkDevice class with mocks."""
    
    @patch('netssh_rs.PyNetworkDevice.create')
    def test_device_creation(self, mock_create):
        """Test device creation."""
        config = netssh_rs.PyDeviceConfig(
            device_type="cisco_ios",
            host="192.168.1.1",
            username="admin",
            password="password"
        )
        
        # Configure the mock
        mock_create.return_value = "mock_device"
        
        # Call the function
        result = netssh_rs.PyNetworkDevice.create(config)
        
        # Assert the mock was called with the right arguments
        mock_create.assert_called_once_with(config)
        self.assertEqual(result, "mock_device")


if __name__ == "__main__":
    unittest.main()