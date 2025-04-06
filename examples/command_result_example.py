#!/usr/bin/env python3
"""
Example of using netssh-rs with CommandResult objects.

This example demonstrates both the standard NetworkDevice API and the
ParallelExecutionManager for executing commands on multiple devices and
working with CommandResult objects.
"""

import sys
import os
import json
from typing import List, Dict

# Add the netssh_rs module to the Python path
sys.path.append(os.path.join(os.path.dirname(__file__), '..'))

try:
    # noqa comments to ignore import warnings related to dynamic imports
    from netssh_rs import DeviceConfig, NetworkDevice, ParallelExecutionManager, CommandResult, BatchCommandResults  # noqa
except ImportError:
    print("Error: Could not import netssh_rs module.")
    print("Make sure you've built the Rust library with 'cargo build'.")
    sys.exit(1)

# Initialize logging
from netssh_rs import initialize_logging
initialize_logging(debug=True, console=True)


def single_device_example():
    """Example of using NetworkDevice with CommandResult objects."""
    print("\n=== Single Device Example ===")
    
    # Create device configuration
    config = DeviceConfig(
        device_type="cisco_ios",
        host="192.168.1.1",
        username="admin",
        password="password"
    )
    
    try:
        # Create and connect to the device
        device = NetworkDevice.create(config)
        device.connect()
        
        # Send a command and get the CommandResult
        result = device.send_command("show version")
        
        # Work with the CommandResult object
        print(f"Command: {result.command}")
        print(f"Status: {result.status}")
        print(f"Duration: {result.duration_ms} ms")
        
        if result.is_success():
            print("Command executed successfully!")
            # Print first 5 lines of output
            if result.output:
                lines = result.output.split('\n')
                print("\nOutput (first 5 lines):")
                for line in lines[:5]:
                    print(f"  {line}")
        else:
            print(f"Command failed: {result.error}")
        
        # Convert to dictionary
        result_dict = result.to_dict()
        print("\nResult as dictionary:")
        print(json.dumps(result_dict, indent=2))
        
        # Enter config mode (returns CommandResult)
        config_result = device.enter_config_mode()
        print(f"\nEntered config mode: {config_result.is_success()}")
        
        # Send config commands (returns CommandResult)
        config_set_result = device.send_config_set(["hostname ROUTER", "interface GigabitEthernet0/1", "description TEST"])
        print(f"Config commands sent: {config_set_result.is_success()}")
        print(f"Duration: {config_set_result.duration_ms} ms")
        
        # Exit config mode (returns CommandResult)
        exit_result = device.exit_config_mode()
        print(f"Exited config mode: {exit_result.is_success()}")
        
        # Close the connection
        device.close()
        
    except Exception as e:
        print(f"Error: {e}")


def parallel_execution_example():
    """Example of using ParallelExecutionManager with CommandResult objects."""
    print("\n=== Parallel Execution Example ===")
    
    # Create device configurations
    configs = [
        DeviceConfig(
            device_type="cisco_ios",
            host="192.168.1.1",
            username="admin",
            password="password"
        ),
        DeviceConfig(
            device_type="cisco_ios",
            host="192.168.1.2",
            username="admin",
            password="password"
        ),
        DeviceConfig(
            device_type="arista_eos",
            host="192.168.1.3",
            username="admin",
            password="password"
        )
    ]
    
    try:
        # Create the parallel execution manager
        manager = ParallelExecutionManager(
            max_concurrency=2,
            command_timeout_seconds=30,
            failure_strategy="skip_device"
        )
        
        # Execute a command on all devices
        results = manager.execute_command_on_all(configs, "show version")
        
        # Work with BatchCommandResults
        print(f"Device count: {results.device_count}")
        print(f"Command count: {results.command_count}")
        print(f"Success count: {results.success_count}")
        print(f"Failure count: {results.failure_count}")
        print(f"Total duration: {results.duration_ms} ms")
        
        # Get all results
        all_results = results.get_all_results()
        print(f"\nTotal results: {len(all_results)}")
        
        # Get successful results
        successful = results.get_successful_results()
        print(f"Successful results: {len(successful)}")
        
        # Get failed results
        failed = results.get_failed_results()
        print(f"Failed results: {len(failed)}")
        
        # Process individual results
        print("\nResults by device:")
        for device_id in [cfg.host for cfg in configs]:
            device_results = results.get_device_results(device_id)
            if device_results:
                print(f"\n  Device: {device_id}")
                for result in device_results:
                    print(f"    Command: {result.command}")
                    print(f"    Status: {result.status}")
                    print(f"    Duration: {result.duration_ms} ms")
                    if result.output:
                        output_preview = result.output.split('\n')[0]
                        print(f"    Output preview: {output_preview}")
        
        # Compare outputs across devices
        print("\nComparing outputs:")
        comparison = results.compare_outputs("show version")
        for device_id, output in comparison.items():
            # Fix the backslash issue by using string formatting directly
            line_count = len(output.split('\n'))
            print(f"  {device_id}: {line_count} lines of output")
        
        # Convert to different formats
        print("\nResults as JSON available with results.to_json()")
        print("Results as CSV available with results.to_csv()")
        print("Results as table available with results.format_as_table()")
        
        # Cleanup
        manager.cleanup()
        
    except Exception as e:
        print(f"Error: {e}")


def main():
    """Main function."""
    print("netssh-rs CommandResult Example")
    print("================================")
    
    # Single device example
    single_device_example()
    
    # Parallel execution example
    parallel_execution_example()


if __name__ == "__main__":
    main() 