#!/usr/bin/env python3
"""
Example demonstrating parallel command execution with netssh-rs.
This example shows how to:
1. Configure multiple devices
2. Execute the same command on all devices in parallel
3. Execute multiple commands sequentially on all devices in parallel
4. Execute different commands on different devices
5. Process and format the results
6. Proper resource management
7. Error handling and propagation
8. Progress reporting
9. Timeout verification
"""

import netssh_rs
import time
import signal
import sys
from concurrent.futures import ThreadPoolExecutor

# Global variable to hold manager reference for signal handler
_manager = None

def signal_handler(sig, frame):
    """Handle CTRL+C by cleaning up connections before exit"""
    print("\nInterrupt received, cleaning up connections...")
    if _manager is not None:
        _manager.cleanup()
    sys.exit(0)

def progress_callback(completed, total, device_id=None, command=None, status=None):
    """Callback function to report progress of parallel operations"""
    percent = (completed / total) * 100
    if device_id and command:
        status_text = f" - Device: {device_id}, Command: {command}, Status: {status}"
    else:
        status_text = ""
    print(f"\rProgress: [{completed}/{total}] {percent:.1f}%{status_text}", end="")
    if completed == total:
        print()  # Add newline at the end

def main():
    # Set up signal handler for graceful termination
    signal.signal(signal.SIGINT, signal_handler)
    
    # Initialize logging
    netssh_rs.initialize_logging(debug=False, console=True)
    
    # Create device configurations
    devices = [
        netssh_rs.PyDeviceConfig(
            device_type="cisco_ios",
            host="192.168.1.25",
            username="admin",
            password="moimran@123",
            port=22,
            timeout_seconds=30,
            secret="moimran@123",
            session_log=None
        ),
        netssh_rs.PyDeviceConfig(
            device_type="cisco_ios",
            host="192.168.1.125",
            username="admin",
            password="moimran@123",
            port=22,
            timeout_seconds=30,
            secret="moimran@123",
            session_log=None
        ),
    ]
    
    # Create a parallel execution manager with improved settings
    manager = netssh_rs.PyParallelExecutionManager(
        max_concurrency=10,
        command_timeout_seconds=60,
        connection_timeout_seconds=30,
        failure_strategy="continue_device",  # Continue with other commands on the same device if one fails
        reuse_connections=True
    )
    
    # Set global reference for signal handler
    global _manager
    _manager = manager
    
    # Track progress manually since progress_callback isn't supported
    total_operations = len(devices)
    completed = 0
    
    try:
        # Example 1: Execute the same command on all devices
        print("\n=== Example 1: Execute the same command on all devices ===")
        start_time = time.time()
        
        # Display manual progress information
        print(f"Executing 'show version' on {len(devices)} devices...")
        
        # Execute the command
        results = manager.execute_command_on_all(devices, "show version")
        
        # Update progress
        print(f"Completed: {len(devices)}/{len(devices)} devices (100%)")
        
        end_time = time.time()
        print(f"Execution time: {end_time - start_time:.2f} seconds")
        print(f"Results summary: {results.device_count} devices, {results.command_count} commands")
        print(f"Success: {results.success_count}, Failed: {results.failure_count}, Timeout: {results.timeout_count}")
        
        # Print results in table format
        print("\nResults table:")
        print(results.format_as_table())
        
        # Display the actual command output
        print("\nCommand outputs:")
        for result in results.get_command_results("show version"):
            print(f"\n--- Device: {result.device_id} ---")
            print(result.output)
        
        # Example 2: Error handling demonstration
        print("\n=== Example 2: Error handling demonstration ===")
        
        # Display manual progress information
        print(f"Executing invalid command on {len(devices)} devices...")
        
        # Intentionally use an invalid command to demonstrate error handling
        invalid_results = manager.execute_command_on_all(devices, "show invalid command")
        
        # Update progress
        print(f"Completed: {len(devices)}/{len(devices)} devices (100%)")
        
        print(f"Results summary: {invalid_results.device_count} devices, {invalid_results.command_count} commands")
        print(f"Success: {invalid_results.success_count}, Failed: {invalid_results.failure_count}, Timeout: {invalid_results.timeout_count}")
        
        # Display detailed error information
        print("\nDetailed error information:")
        for result in invalid_results.get_failed_results():
            print(f"Device: {result.device_id}, Command: {result.command}")
            print(f"Error: {result.error}")
            print(f"Status: {result.status}")
            print(f"Duration: {result.duration_ms}ms")
            print()
        
        # Example 3: Timeout verification
        print("\n=== Example 3: Timeout verification ===")
        
        # Create a device with very short timeout to test timeout handling
        short_timeout_device = netssh_rs.PyDeviceConfig(
            device_type="cisco_ios",
            host="192.168.1.25",
            username="admin",
            password="moimran@123",
            port=22,
            timeout_seconds=1,  # Very short timeout
            secret="moimran@123",
            session_log=None
        )
        
        # Display manual progress information
        print("Executing command with very short timeout...")
        
        # Execute a command that might take longer than the timeout
        timeout_results = manager.execute_command_on_all([short_timeout_device], "show running-config")
        
        # Update progress
        print("Completed: 1/1 devices (100%)")
        
        print(f"Results summary: {timeout_results.device_count} devices, {timeout_results.command_count} commands")
        print(f"Success: {timeout_results.success_count}, Failed: {timeout_results.failure_count}, Timeout: {timeout_results.timeout_count}")
        
        # Display timeout information
        print("\nTimeout information:")
        for result in timeout_results.get_command_results():
            print(f"Device: {result.device_id}, Command: {result.command}")
            print(f"Duration before timeout: {result.duration_ms}ms")
            print()
        
        # Example 4: Resource management demonstration
        print("\n=== Example 4: Resource management demonstration ===")
        
        # Demonstrate connection pooling and resource management
        print("Executing multiple command sets with connection reuse...")
        
        # First set of commands
        cmd_set1 = ["show version", "show interfaces brief"]
        print(f"Executing command set 1 ({len(cmd_set1)} commands) on {len(devices)} devices...")
        results1 = manager.execute_commands_on_all(devices, cmd_set1)
        print(f"Completed: {len(devices)}/{len(devices)} devices (100%)")
        print(f"Command set 1 completed: {results1.success_count} successes, {results1.failure_count} failures")
        
        # Second set of commands (reusing the same connections)
        cmd_set2 = ["show ip route summary", "show users"]
        print(f"Executing command set 2 ({len(cmd_set2)} commands) on {len(devices)} devices...")
        results2 = manager.execute_commands_on_all(devices, cmd_set2)
        print(f"Completed: {len(devices)}/{len(devices)} devices (100%)")
        print(f"Command set 2 completed: {results2.success_count} successes, {results2.failure_count} failures")
        
        # Verify connection reuse
        print("\nConnection statistics:")
        conn_stats = manager.get_connection_stats()
        print(f"Total connections created: {conn_stats.get('total_created', 'N/A')}")
        print(f"Connections reused: {conn_stats.get('reused', 'N/A')}")
        print(f"Currently active connections: {conn_stats.get('active', 'N/A')}")
        print(f"Failed connections: {conn_stats.get('failed', 'N/A')}")
        
    finally:
        # Ensure proper cleanup of resources
        print("\nCleaning up resources...")
        manager.cleanup()
        print("All connections closed.")

if __name__ == "__main__":
    main() 