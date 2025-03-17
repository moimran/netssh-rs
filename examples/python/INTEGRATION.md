# Integrating netssh_rs with Python Backend Code

This guide explains how to integrate the netssh_rs Python bindings with your existing Python backend code.

## Prerequisites

- Python 3.7 or higher
- Rust toolchain (for building the bindings)
- Maturin (for building the Python package)

## Installation

1. Install the netssh_rs Python package:

```bash
# From the netssh-rs directory
make setup
make develop
```

Or manually:

```bash
cd netssh-rs
uv pip install maturin
maturin develop
```

2. Install required Python dependencies:

```bash
uv pip install -r python/requirements.txt
```

## Basic Integration

Here's a simple example of how to integrate netssh_rs with your Python code:

```python
import netssh_rs

# Initialize logging
netssh_rs.initialize_logging(debug=True, console=True)

def connect_to_device(device_type, host, username, password, port=22):
    """Connect to a network device and return the device object."""
    config = netssh_rs.PyDeviceConfig(
        device_type=device_type,
        host=host,
        username=username,
        password=password,
        port=port,
        timeout_seconds=60
    )
    
    device = netssh_rs.PyNetworkDevice.create(config)
    device.connect()
    return device

def get_device_info(device):
    """Get basic information from a device."""
    version = device.send_command("show version")
    interfaces = device.send_command("show ip interface brief")
    
    return {
        "version": version,
        "interfaces": interfaces
    }

# Example usage
if __name__ == "__main__":
    device = connect_to_device(
        device_type="cisco_ios",
        host="192.168.1.1",
        username="admin",
        password="password"
    )
    
    try:
        info = get_device_info(device)
        print(info)
    finally:
        device.close()
```

## Integration with FastAPI

For a more complete example of integrating with a FastAPI backend, see the `backend_integration_example.py` file.

Key points for FastAPI integration:

1. **Run device operations in a thread pool**: Network operations are blocking, so run them in a thread pool to avoid blocking the event loop:

```python
@app.post("/api/device/command")
async def execute_command(request: CommandRequest):
    loop = asyncio.get_event_loop()
    result = await loop.run_in_executor(
        None,  # Use default executor
        run_device_command,  # Your function that uses netssh_rs
        request.device,
        request.command
    )
    return result
```

2. **Handle errors properly**: Rust errors are converted to Python RuntimeError exceptions:

```python
try:
    device = netssh_rs.PyNetworkDevice.create(config)
    device.connect()
    # ...
except RuntimeError as e:
    logger.error(f"Device connection error: {str(e)}")
    raise HTTPException(status_code=500, detail=str(e))
```

3. **Always close connections**: Use context managers or try/finally blocks to ensure connections are closed:

```python
# Using context manager
with netssh_rs.PyNetworkDevice.create(config) as device:
    device.connect()
    result = device.send_command(command)
    return result

# Using try/finally
device = netssh_rs.PyNetworkDevice.create(config)
try:
    device.connect()
    result = device.send_command(command)
    return result
finally:
    device.close()
```

## Integration with SQLAlchemy Models

If you're using SQLAlchemy models to store device information, you can easily integrate with netssh_rs:

```python
from sqlalchemy.orm import Session
from your_app.models import Device

def connect_to_db_device(db: Session, device_id: int):
    """Connect to a device stored in the database."""
    # Get device from database
    db_device = db.query(Device).filter(Device.id == device_id).first()
    if not db_device:
        raise ValueError(f"Device with ID {device_id} not found")
    
    # Create netssh_rs config
    config = netssh_rs.PyDeviceConfig(
        device_type=db_device.device_type,
        host=db_device.host,
        username=db_device.username,
        password=db_device.password,
        port=db_device.port,
        timeout_seconds=60,
        secret=db_device.enable_secret
    )
    
    # Connect to device
    device = netssh_rs.PyNetworkDevice.create(config)
    device.connect()
    return device
```

## Performance Considerations

The netssh_rs Python bindings provide significant performance improvements over pure Python implementations:

1. **Connection handling**: Faster connection establishment and command execution
2. **Memory usage**: Lower memory footprint for large outputs
3. **Concurrency**: Better handling of multiple concurrent connections

For best performance:

- Reuse device connections when possible
- Use connection pooling for frequently accessed devices
- Run network operations in a thread pool to avoid blocking the event loop

## Error Handling

All netssh_rs errors are converted to Python RuntimeError exceptions. You should catch these exceptions and handle them appropriately:

```python
try:
    device = netssh_rs.PyNetworkDevice.create(config)
    device.connect()
    result = device.send_command(command)
except RuntimeError as e:
    logger.error(f"Error: {str(e)}")
    # Handle the error
```

## Logging

netssh_rs provides its own logging system that can be initialized with:

```python
netssh_rs.initialize_logging(debug=True, console=True)
```

You can also integrate with Python's logging system:

```python
import logging

# Configure Python logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Initialize netssh_rs logging
netssh_rs.initialize_logging(debug=True, console=True)

# Log from Python
logger.info("Connecting to device...")

# netssh_rs will log to its own files and console if enabled
```

## Thread Safety

The netssh_rs Python bindings are thread-safe, but each device connection should be used by only one thread at a time. If you need to access a device from multiple threads, create separate connections or use a connection pool with proper locking.