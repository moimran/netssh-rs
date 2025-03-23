#!/usr/bin/env python3
"""
Example demonstrating parallel command execution with netssh-rs.
This example shows how to:
1. Configure multiple devices
2. Execute the same command on all devices in parallel
3. Execute multiple commands sequentially on all devices in parallel
4. Execute different commands on different devices
5. Process and format the results
"""

import netssh_rs
import time

def main():
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
    
    # Create a parallel execution manager
    manager = netssh_rs.PyParallelExecutionManager(
        max_concurrency=10,
        command_timeout_seconds=60,
        connection_timeout_seconds=30,
        failure_strategy="continue_device",
        reuse_connections=True
    )
    
    # Example 1: Execute the same command on all devices
    print("\n=== Example 1: Execute the same command on all devices ===")
    start_time = time.time()
    
    # Execute the command
    results = manager.execute_command_on_all(devices, "show version")
    
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
    
    # # Example 2: Execute multiple commands sequentially on all devices in parallel
    # print("\n=== Example 2: Execute multiple commands sequentially on all devices ===")
    # start_time = time.time()
    
    # # Define commands to execute
    # commands = [
    #     "show version",
    #     "show interfaces brief",
    #     "show ip route summary"
    # ]
    
    # # Execute the commands
    # results = manager.execute_commands_on_all(devices, commands)
    
    # end_time = time.time()
    # print(f"Execution time: {end_time - start_time:.2f} seconds")
    # print(f"Results summary: {results.device_count} devices, {results.command_count} commands")
    # print(f"Success: {results.success_count}, Failed: {results.failure_count}, Timeout: {results.timeout_count}")
    
    # # Print results in table format
    # print("\nResults table:")
    # print(results.format_as_table())
    
    # # Example 3: Execute different commands on different devices
    # print("\n=== Example 3: Execute different commands on different devices ===")
    # start_time = time.time()
    
    # # Define device-specific commands
    # device_commands = {
    #     devices[0]: ["show version", "show interfaces"],
    #     devices[1]: ["show ip route", "show running-config"],
    #     devices[2]: ["show system information", "show interfaces terse"]
    # }
    
    # # Execute the commands
    # results = manager.execute_commands(device_commands)
    
    # end_time = time.time()
    # print(f"Execution time: {end_time - start_time:.2f} seconds")
    # print(f"Results summary: {results.device_count} devices, {results.command_count} commands")
    # print(f"Success: {results.success_count}, Failed: {results.failure_count}, Timeout: {results.timeout_count}")
    
    # # Print results in table format
    # print("\nResults table:")
    # print(results.format_as_table())
    
    # # Example 4: Process results in different formats
    # print("\n=== Example 4: Process results in different formats ===")
    
    # # Get all results for a specific device
    # device1_results = results.get_device_results("192.168.1.1")
    # if device1_results:
    #     print(f"\nCommands executed on 192.168.1.1: {len(device1_results)}")
    #     for result in device1_results:
    #         print(f"  - {result.command} (Status: {result.status}, Duration: {result.duration_ms}ms)")
    
    # # Get results for a specific command across all devices
    # version_results = results.get_command_results("show version")
    # print(f"\nDevices that executed 'show version': {len(version_results)}")
    # for result in version_results:
    #     print(f"  - {result.device_id} (Status: {result.status}, Duration: {result.duration_ms}ms)")
    
    # # Get successful results
    # successful_results = results.get_successful_results()
    # print(f"\nSuccessful commands: {len(successful_results)}")
    
    # # Get failed results
    # failed_results = results.get_failed_results()
    # print(f"\nFailed commands: {len(failed_results)}")
    # for result in failed_results:
    #     print(f"  - {result.device_id}: {result.command} - Error: {result.error}")
    
    # # Export results to JSON
    # json_results = results.to_json()
    # print("\nJSON Results (sample):")
    # print(json_results[:500] + "...")  # Show truncated sample
    
    # # Export results to CSV
    # csv_results = results.to_csv()
    # print("\nCSV Results (sample):")
    # csv_lines = csv_results.split("\n")
    # print("\n".join(csv_lines[:5]) + "\n...")  # Show first few lines
    
    # # Compare outputs for the same command across devices
    # output_comparison = results.compare_outputs("show version")
    # print("\nOutput comparison for 'show version':")
    # for output, devices in output_comparison.items():
    #     print(f"  - Output variant found on {len(devices)} devices: {', '.join(devices)}")
    
    # # Clean up connections
    # manager.cleanup()
    # print("\nConnections cleaned up.")

if __name__ == "__main__":
    main() 