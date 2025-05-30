# 🚀 Job Scheduler

A powerful Rust-based job scheduler application with SSH execution capabilities, built using Apalis and netssh-rs.

## ✨ Features

- **Multiple Job Types**: Ad-hoc, one-time scheduled, and recurring jobs with cron expressions
- **SSH Command Execution**: Execute commands on remote network devices using netssh-rs
- **SQLite Storage**: Lightweight, file-based storage with easy migration to other backends
- **Web Dashboard**: Built-in web UI for monitoring and managing jobs
- **REST API**: Complete API for job management and monitoring
- **Device Support**: Cisco IOS, IOS-XE, NX-OS, ASA, XR, Arista EOS, Juniper JUNOS
- **Extensible Architecture**: Easy to add new job types and storage backends

## 🏗️ Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Web UI        │    │   REST API      │    │   Apalis        │
│   (Dashboard)   │    │   (Job Mgmt)    │    │   (Scheduler)   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
         ┌─────────────────────────────────────────────────┐
         │              Storage Layer                      │
         │            (SQLite/Abstract)                    │
         └─────────────────────────────────────────────────┘
                                 │
         ┌─────────────────────────────────────────────────┐
         │              SSH Execution                      │
         │              (netssh-rs)                        │
         └─────────────────────────────────────────────────┘
```

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+ 
- SQLite (included)

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd scheduler
```

2. Build the application:
```bash
cargo build --release
```

3. Run the application:
```bash
cargo run
```

4. Access the web dashboard:
```
http://localhost:8080/board
```

## 📖 Usage

### Configuration

Create a `config.toml` file or use environment variables:

```toml
[database]
url = "sqlite:scheduler.db"
max_connections = 10

[server]
host = "127.0.0.1"
port = 8080

[worker]
concurrency = 4
timeout_seconds = 300

[board]
enabled = true
ui_path = "/board"
api_prefix = "/board/api"
auth_enabled = false

[logging]
level = "info"
format = "text"  # or "json" for structured logging
file = "scheduler.log"  # optional file output with rotation
```

### Environment Variables

You can also configure using environment variables with the `SCHEDULER_` prefix:

```bash
export SCHEDULER_DATABASE_URL="sqlite:scheduler.db"
export SCHEDULER_SERVER_PORT=8080
export SCHEDULER_WORKER_CONCURRENCY=4
export SCHEDULER_LOGGING_LEVEL=debug
export SCHEDULER_LOGGING_FORMAT=json
export SCHEDULER_LOGGING_FILE=scheduler.log
```

### Creating SSH Jobs

#### Via REST API

```bash
# Create an ad-hoc SSH job
curl -X POST http://localhost:8080/api/jobs \
  -H "Content-Type: application/json" \
  -d '{
    "job_type": "SshCommand",
    "payload": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "connection": {
        "host": "192.168.1.1",
        "username": "admin",
        "password": "password",
        "device_type": "cisco_ios",
        "port": 22,
        "timeout_seconds": 60
      },
      "commands": [
        "show version",
        "show ip interface brief"
      ],
      "description": "Device information gathering"
    }
  }'
```

#### Scheduled Jobs

```bash
# Schedule a job for later execution
curl -X POST http://localhost:8080/api/jobs \
  -H "Content-Type: application/json" \
  -d '{
    "job_type": "SshCommand",
    "scheduled_for": "2024-01-01T12:00:00Z",
    "payload": { ... }
  }'
```

#### Recurring Jobs

```bash
# Create a recurring job with cron expression
curl -X POST http://localhost:8080/api/jobs \
  -H "Content-Type: application/json" \
  -d '{
    "job_type": "SshCommand",
    "cron_expression": "0 0 * * *",
    "payload": { ... }
  }'
```

### Monitoring Jobs

#### Web Dashboard

Visit `http://localhost:8080/board` to access the web dashboard where you can:

- View job statistics and status
- Monitor running jobs in real-time
- Browse job history and results
- View detailed logs for each job

#### REST API

```bash
# List all jobs
curl http://localhost:8080/api/jobs

# Get specific job details
curl http://localhost:8080/api/jobs/{job_id}

# Get job logs
curl http://localhost:8080/api/jobs/{job_id}/logs

# Health check
curl http://localhost:8080/api/health
```

## 📋 Comprehensive Logging

The scheduler includes industry-standard structured logging with the `tracing` crate, providing detailed insights into job execution, system performance, and debugging information.

### Logging Features

- **Structured Logging**: JSON and text formats with contextual fields
- **Multiple Output Options**: Console, file, or both simultaneously
- **File Rotation**: Daily log rotation with automatic cleanup
- **Performance Metrics**: Execution times, queue sizes, and throughput
- **Contextual Information**: Job IDs, SSH hosts, command details, and error context
- **Configurable Levels**: TRACE, DEBUG, INFO, WARN, ERROR

### Configuration

Configure logging via `config.toml` or environment variables:

```toml
[logging]
level = "info"           # Log level: trace, debug, info, warn, error
format = "json"          # Output format: "text" or "json"
file = "scheduler.log"   # Optional file output with daily rotation
```

Environment variables:
```bash
export SCHEDULER_LOGGING_LEVEL=debug
export SCHEDULER_LOGGING_FORMAT=json
export SCHEDULER_LOGGING_FILE=logs/scheduler.log
```

### Structured Log Fields

The scheduler automatically includes contextual fields in log entries:

**Job Context:**
- `job_id`: Unique job identifier
- `host`: SSH target hostname
- `device_type`: Network device type
- `command_count`: Number of commands to execute
- `duration_ms`: Execution time in milliseconds

**SSH Context:**
- `ssh_host`: Target device hostname
- `username`: SSH username
- `port`: SSH port number
- `command`: Executed command
- `output_length`: Command output size

**API Context:**
- `http_method`: HTTP request method
- `http_path`: API endpoint path
- `status_code`: HTTP response status

**Database Context:**
- `db_operation`: Database operation type
- `affected_rows`: Number of affected records

### Log Examples

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

### Log Aggregation

For production deployments, use JSON format with log aggregation tools:

- **ELK Stack**: Elasticsearch, Logstash, Kibana
- **Grafana Loki**: Lightweight log aggregation
- **Fluentd/Fluent Bit**: Log collection and forwarding
- **Datadog/New Relic**: Cloud-based monitoring

### Performance Monitoring

The scheduler logs performance metrics for monitoring:

```json
{
  "level": "INFO",
  "message": "SSH job completed successfully",
  "job_id": "550e8400-e29b-41d4-a716-446655440000",
  "duration_ms": 1250,
  "command_count": 3,
  "successful_commands": 3,
  "failed_commands": 0
}
```

## 🔧 Development

### Running Examples

```bash
# Basic usage example
cargo run --example basic_usage

# Web UI example
cargo run --example with_board

# Logging demo
cargo run --example logging_demo
```

### Testing

```bash
# Run all tests
cargo test

# Run with logging
RUST_LOG=debug cargo test
```

### Adding New Job Types

1. Define your job payload in `src/jobs/types.rs`
2. Implement the job handler in `src/jobs/`
3. Add the job type to the `JobType` enum
4. Update the API handlers to support the new job type

## 🔌 Supported Devices

- **Cisco IOS** (`cisco_ios`)
- **Cisco IOS-XE** (`cisco_ios_xe`) 
- **Cisco NX-OS** (`cisco_nxos`)
- **Cisco ASA** (`cisco_asa`)
- **Cisco IOS-XR** (`cisco_xr`)
- **Arista EOS** (`arista_eos`)
- **Juniper JUNOS** (`juniper_junos`)

## 📊 API Reference

### Job Management

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/jobs` | Create a new job |
| GET | `/api/jobs` | List jobs with filtering |
| GET | `/api/jobs/{id}` | Get job details |
| DELETE | `/api/jobs/{id}` | Delete a job |
| GET | `/api/jobs/{id}/logs` | Get job logs |

### Connection Profiles

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/connections` | Create connection profile |
| GET | `/api/connections` | List connection profiles |
| GET | `/api/connections/{id}` | Get connection profile |
| PUT | `/api/connections/{id}` | Update connection profile |
| DELETE | `/api/connections/{id}` | Delete connection profile |

## 🛠️ Storage Backends

The application uses an abstract storage layer that currently supports:

- **SQLite** (default) - File-based, zero-configuration
- **Future**: PostgreSQL, MySQL, Redis

To migrate to a different backend, implement the `Storage` trait in `src/storage/traits.rs`.

## 🔒 Security Considerations

- Passwords are not stored in connection profiles
- Use environment variables for sensitive configuration
- Enable authentication in production (`board.auth_enabled = true`)
- Consider using SSH keys instead of passwords
- Run with minimal privileges

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## 📝 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- [Apalis](https://github.com/geofmureithi/apalis) - Job scheduling framework
- [netssh-rs](https://github.com/moimran/netssh-rs) - SSH connectivity library
- [SQLx](https://github.com/launchbadge/sqlx) - Async SQL toolkit
- [Axum](https://github.com/tokio-rs/axum) - Web framework
