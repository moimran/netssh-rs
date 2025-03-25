#!/usr/bin/env python3
"""
Test file to verify that VSCode IntelliSense is working correctly with netssh_rs.
"""

import sys
import logging
from typing import Dict, Any, List

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(levelname)s: %(message)s')
logger = logging.getLogger(__name__)

# Import modules
try:
    # First try direct import (this is what should work after our __init__.py fix)
    from netssh_rs import (
        PyDeviceConfig, 
        PyNetworkDevice,
        PyDeviceInfo,
        PyCommandResult,
        PyBatchCommandResults,
        PyParallelExecutionManager,
        initialize_logging
    )
    logger.info("Successfully imported from netssh_rs")
except ImportError as e:
    logger.error(f"Error importing from netssh_rs: {e}")
    
    try:
        # Fall back to nested import
        from netssh_rs import (
            PyDeviceConfig, 
            PyNetworkDevice,
            PyDeviceInfo,
            PyCommandResult,
            PyBatchCommandResults,
            PyParallelExecutionManager,
            initialize_logging
        )
        logger.info("Successfully imported from netssh_rs.netssh_rs")
    except ImportError as e:
        logger.error(f"Error importing from netssh_rs.netssh_rs: {e}")
        sys.exit(1)

def intellisense_test() -> None:
    """Test IntelliSense functionality."""
    
    # Initialize logging
    initialize_logging(debug=True, console=True)
    
    # Create device configuration
    config = PyDeviceConfig(
        device_type="cisco_ios",
        host="192.168.1.1",
        username="admin",
        password="password",
        port=22,
        timeout_seconds=60,
        secret="enable_secret",
        session_log="session.log"
    )
    
    # Access configuration properties
    device_type = config.device_type
    host = config.host
    username = config.username
    
    # Create network device
    device = PyNetworkDevice.create(config)
    
    # Use device methods
    device.connect()
    device.terminal_settings()
    
    # Check config mode
    is_config_mode = device.check_config_mode()
    
    # Send commands
    output = device.send_command("show version")
    outputs = device.send_commands(["show ip interface brief", "show vlan"])
    
    # Get device info
    device_info = device.get_device_info()
    vendor = device_info.vendor
    model = device_info.model
    hostname = device_info.hostname
    
    # Parallel execution
    parallel = PyParallelExecutionManager(max_threads=5, timeout=30)
    parallel.add_device(config)
    
    # Create multiple configs
    configs: List[PyDeviceConfig] = [
        PyDeviceConfig(
            device_type="cisco_ios",
            host="192.168.1.2",
            username="admin",
            password="password"
        ),
        PyDeviceConfig(
            device_type="cisco_ios",
            host="192.168.1.3",
            username="admin",
            password="password"
        )
    ]
    
    # Add multiple devices
    parallel.add_devices(configs)
    
    # Execute commands
    results = parallel.execute_command("show version")
    batch_results = parallel.execute_commands(["show ip interface brief", "show vlan"])
    
    # Process results
    for device_id, result in results.items():
        command = result.command
        output = result.output
        status = result.status
        execution_time = result.execution_time
        
    # Process batch results
    for device_id, batch in batch_results.items():
        success = batch.successful
        total_time = batch.total_execution_time
        
        # Access command results
        cmd_results = batch.results
        for cmd_result in cmd_results:
            cmd = cmd_result.command
            cmd_output = cmd_result.output
            
        # Get failed commands
        failed_commands = batch.get_failed_commands()
        
    # Close device connection
    device.close()
    
    logger.info("IntelliSense test complete")

def main() -> int:
    """Run the test."""
    
    logger.info("NetSSH-RS IntelliSense Test")
    logger.info("-" * 50)
    
    # This file is just for IntelliSense testing, not actual execution
    logger.info("This file is for IntelliSense testing only")
    logger.info("It's not meant to be executed - it demonstrates IDE code completion")
    
    # Only run the test in "execute" mode
    if len(sys.argv) > 1 and sys.argv[1] == "--execute":
        try:
            intellisense_test()
        except Exception as e:
            logger.error(f"Error during test: {e}")
            return 1
    
    logger.info("-" * 50)
    logger.info("Test complete")
    
    return 0

if __name__ == "__main__":
    sys.exit(main())