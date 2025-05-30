# Comprehensive Logging Implementation

This document describes the comprehensive logging system implemented for the Rust Job Scheduler project using industry best practices.

## Overview

The scheduler now includes a robust, structured logging system built on the `tracing` crate, providing detailed insights into job execution, system performance, and debugging information across all components.

## Features Implemented

### 🔧 Core Logging Infrastructure

- **Centralized Logging Module** (`src/logging.rs`)
  - Configurable initialization with multiple output formats
  - Support for both console and file output with daily rotation
  - Environment-based configuration
  - Graceful fallback handling

- **Enhanced Dependencies**
  - `tracing`: Modern structured logging framework
  - `tracing-subscriber`: Output formatting and filtering
  - `tracing-appender`: File rotation and non-blocking I/O

### 📊 Structured Logging Fields

All log entries include contextual structured fields for easy filtering and analysis:

**Job Context:**
- `job_id`: Unique job identifier
- `host`: SSH target hostname  
- `device_type`: Network device type (cisco_ios, juniper, etc.)
- `command_count`: Number of commands to execute
- `duration_ms`: Execution time in milliseconds
- `retry_count`: Current retry attempt

**SSH Context:**
- `ssh_host`: Target device hostname
- `username`: SSH username
- `port`: SSH port number
- `command`: Executed command
- `output_length`: Command output size in bytes

**API Context:**
- `http_method`: HTTP request method (GET, POST, etc.)
- `http_path`: API endpoint path
- `status_code`: HTTP response status

**Database Context:**
- `db_operation`: Database operation type
- `affected_rows`: Number of affected records
- `query`: SQL query (in debug mode)

### 🎯 Log Levels Implementation

- **ERROR**: Critical failures preventing job execution
- **WARN**: Recoverable issues (job retries, connection timeouts)
- **INFO**: Important operational events (job lifecycle, system status)
- **DEBUG**: Detailed execution flow and state changes
- **TRACE**: Very detailed debugging information

### 📁 Output Formats

**Text Format (Development):**
```
2024-01-15T10:30:45.123Z INFO scheduler::jobs::ssh_job: Starting SSH job execution job_id=550e8400-e29b-41d4-a716-446655440000 host=192.168.1.1 device_type=cisco_ios command_count=3
```

**JSON Format (Production):**
```json
{
  "timestamp": "2024-01-15T10:30:45.123Z",
  "level": "INFO",
  "target": "scheduler::jobs::ssh_job",
  "message": "Starting SSH job execution",
  "job_id": "550e8400-e29b-41d4-a716-446655440000",
  "host": "192.168.1.1",
  "device_type": "cisco_ios",
  "command_count": 3,
  "timeout_seconds": 300,
  "retry_count": 0
}
```

## Components Enhanced with Logging

### 🔄 Job Lifecycle Logging

**SSH Job Handler** (`src/jobs/ssh_job.rs`):
- Job start/completion with timing metrics
- SSH connection establishment and teardown
- Individual command execution with output metrics
- Error handling with detailed context
- Performance metrics (execution times, success rates)

### 🌐 API Request/Response Logging

**API Handlers** (`src/api/handlers.rs`):
- Request logging with method, path, and parameters
- Response logging with status codes and timing
- Error logging with detailed context
- Health check monitoring

### 💾 Database Operations Logging

**SQLite Storage** (`src/storage/sqlite.rs`):
- Connection establishment and health checks
- Query execution with performance metrics
- CRUD operations with affected row counts
- Error handling with query context

### 🖥️ Board Service Logging

**Board UI** (`src/board/mod.rs`):
- Route creation and configuration
- UI access logging
- Service status monitoring

### 🚀 Application Lifecycle Logging

**Main Application** (`src/main.rs`):
- Startup sequence with configuration details
- Service initialization status
- Graceful shutdown logging

## Configuration Options

### Configuration File (`config.toml`):
```toml
[logging]
level = "info"           # trace, debug, info, warn, error
format = "json"          # "text" or "json"
file = "scheduler.log"   # Optional file output with daily rotation
```

### Environment Variables:
```bash
export SCHEDULER_LOGGING_LEVEL=debug
export SCHEDULER_LOGGING_FORMAT=json
export SCHEDULER_LOGGING_FILE=logs/scheduler.log
```

## Performance Monitoring

The logging system captures key performance metrics:

- **Job Execution Times**: Start to completion duration
- **Command Performance**: Individual SSH command timing
- **Database Query Performance**: Query execution times
- **API Response Times**: Request processing duration
- **Connection Metrics**: SSH connection establishment time

## Production Deployment

### Log Aggregation Integration

The JSON format is optimized for log aggregation systems:

- **ELK Stack**: Elasticsearch, Logstash, Kibana
- **Grafana Loki**: Lightweight log aggregation
- **Fluentd/Fluent Bit**: Log collection and forwarding
- **Cloud Services**: Datadog, New Relic, AWS CloudWatch

### File Rotation

- **Daily Rotation**: Automatic log file rotation
- **Non-blocking I/O**: Prevents logging from blocking application
- **Configurable Retention**: Easy cleanup of old log files

## Testing

Comprehensive test suite (`tests/logging_test.rs`):
- Logging initialization validation
- Structured logging verification
- Storage operation logging tests
- SSH job validation logging tests

## Examples

### Logging Demo (`examples/logging_demo.rs`):
- Console text format demonstration
- Console JSON format demonstration  
- File logging with rotation demonstration
- Multiple configuration examples

### Basic Usage (`examples/basic_usage.rs`):
- Integration with existing workflows
- Storage operation logging
- Job validation logging

## Monitoring and Alerting

The structured logging enables easy monitoring:

```json
{
  "level": "ERROR",
  "message": "SSH job failed",
  "job_id": "550e8400-e29b-41d4-a716-446655440000",
  "error": "Connection timeout",
  "host": "192.168.1.1",
  "duration_ms": 30000
}
```

Set up alerts on:
- High error rates (`level: "ERROR"`)
- Long execution times (`duration_ms > threshold`)
- Connection failures (`error: "Connection*"`)
- Job queue buildup (`status: "pending"` count)

## Best Practices Implemented

1. **Structured Fields**: Consistent field naming across components
2. **Performance Metrics**: Timing information for all operations
3. **Error Context**: Detailed error information with context
4. **Non-blocking I/O**: Logging doesn't impact application performance
5. **Configurable Levels**: Easy adjustment for different environments
6. **Graceful Degradation**: Fallback logging if configuration fails
7. **Security**: No sensitive data (passwords, keys) in logs

## Future Enhancements

- **Distributed Tracing**: OpenTelemetry integration for microservices
- **Metrics Integration**: Prometheus metrics alongside logs
- **Log Sampling**: High-volume environment optimization
- **Custom Formatters**: Domain-specific log formatting
- **Log Encryption**: Sensitive environment log protection
