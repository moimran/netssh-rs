#!/usr/bin/env python3
"""
Unit tests for netssh_rs Python bindings.
"""

import unittest
import netssh_rs

class TestNetSshRs(unittest.TestCase):
    """Test cases for netssh_rs Python bindings."""
    
    def test_device_config(self):
        """Test creating a device configuration."""
        config = netssh_rs.PyDeviceConfig(
            device_type="cisco_ios",
            host="192.168.1.1",
            username="admin",
            password="password",
            port=22,
            timeout_seconds=60,
            secret="enable",
            session_log="session.log"
        )
        
        self.assertEqual(config.device_type, "cisco_ios")
        self.assertEqual(config.host, "192.168.1.1")
        self.assertEqual(config.username, "admin")
        self.assertEqual(config.password, "password")
        self.assertEqual(config.port, 22)
        self.assertEqual(config.timeout_seconds, 60)
        self.assertEqual(config.secret, "enable")
        self.assertEqual(config.session_log, "session.log")
    
    def test_logging_initialization(self):
        """Test initializing logging."""
        # This should not raise an exception
        try:
            netssh_rs.initialize_logging(debug=True, console=True)
            self.assertTrue(True)
        except Exception as e:
            self.fail(f"initialize_logging raised exception {e}")

if __name__ == "__main__":
    unittest.main() 