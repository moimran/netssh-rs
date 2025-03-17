#!/usr/bin/env python3
"""
Example of integrating netssh_rs with a Python backend application.

This example demonstrates how to use netssh_rs in a FastAPI backend
to provide network device management capabilities.
"""

import asyncio
import logging
from typing import List, Dict, Optional, Any
from fastapi import FastAPI, HTTPException, BackgroundTasks
from pydantic import BaseModel
import netssh_rs

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Initialize netssh_rs logging
netssh_rs.initialize_logging(debug=True, console=True)

# Create FastAPI app
app = FastAPI(title="Network Device Management API")

# Pydantic models for API
class DeviceCredentials(BaseModel):
    device_type: str = "cisco_ios"
    host: str = ""
    username: str
    password: str
    port: Optional[int] = 22
    timeout_seconds: Optional[int] = 60
    secret: Optional[str] = None
    session_log: Optional[str] = None

class CommandRequest(BaseModel):
    device: DeviceCredentials
    command: str

class ConfigCommandsRequest(BaseModel):
    device: DeviceCredentials
    commands: List[str]

class CommandResponse(BaseModel):
    success: bool
    output: str
    error: Optional[str] = None

# Device connection pool
device_pool = {}

# Helper function to run device commands
def execute_device_command(device_config: DeviceCredentials, command: str) -> str:
    """Execute a command on a network device."""
    try:
        # Convert to PyDeviceConfig
        config = netssh_rs.PyDeviceConfig(
            device_type=device_config.device_type,
            host=device_config.host,
            username=device_config.username,
            password=device_config.password,
            port=device_config.port,
            timeout_seconds=device_config.timeout_seconds,
            secret=device_config.secret,
            session_log=device_config.session_log
        )
        
        # Create and connect to device
        device = netssh_rs.PyNetworkDevice.create(config)
        device.connect()
        
        try:
            # Send command and get output
            output = device.send_command(command)
            return output
        finally:
            # Always close the connection
            device.close()
            
    except Exception as e:
        logger.error(f"Error executing command: {str(e)}")
        raise

# Helper function to run configuration commands
def execute_config_commands(device_config: DeviceCredentials, commands: List[str]) -> str:
    """Execute configuration commands on a network device."""
    try:
        # Convert to PyDeviceConfig
        config = netssh_rs.PyDeviceConfig(
            device_type=device_config.device_type,
            host=device_config.host,
            username=device_config.username,
            password=device_config.password,
            port=device_config.port,
            timeout_seconds=device_config.timeout_seconds,
            secret=device_config.secret,
            session_log=device_config.session_log
        )
        
        # Create and connect to device
        device = netssh_rs.PyNetworkDevice.create(config)
        device.connect()
        
        try:
            # Enter config mode
            device.enter_config_mode(None)
            
            # Send each command
            outputs = []
            for cmd in commands:
                output = device.send_command(cmd)
                outputs.append(output)
            
            # Exit config mode
            device.exit_config_mode(None)
            
            # Save configuration
            device.save_configuration()
            
            return "\n".join(outputs)
        finally:
            # Always close the connection
            device.close()
            
    except Exception as e:
        logger.error(f"Error executing config commands: {str(e)}")
        raise

# API endpoints
@app.post("/api/execute/command", response_model=CommandResponse)
async def execute_command(request: CommandRequest, background_tasks: BackgroundTasks):
    """Execute a command on a network device."""
    try:
        # Run the command in a thread pool to avoid blocking
        loop = asyncio.get_event_loop()
        output = await loop.run_in_executor(
            None, 
            execute_device_command, 
            request.device, 
            request.command
        )
        
        return CommandResponse(success=True, output=output)
    except Exception as e:
        return CommandResponse(success=False, output="", error=str(e))

@app.post("/api/execute/config", response_model=CommandResponse)
async def execute_config(request: ConfigCommandsRequest, background_tasks: BackgroundTasks):
    """Execute configuration commands on a network device."""
    try:
        # Run the commands in a thread pool to avoid blocking
        loop = asyncio.get_event_loop()
        output = await loop.run_in_executor(
            None, 
            execute_config_commands, 
            request.device, 
            request.commands
        )
        
        return CommandResponse(success=True, output=output)
    except Exception as e:
        return CommandResponse(success=False, output="", error=str(e))

@app.get("/health")
async def health_check():
    """Health check endpoint."""
    return {"status": "healthy"}

# Run the application with uvicorn
if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)