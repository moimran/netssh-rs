# Job Scheduler Demo & Quick Start Guide

This document provides a comprehensive guide to running and testing the Job Scheduler application.

## 🚀 Quick Start

### Prerequisites

- **Rust** (1.70 or later)
- **Git**
- **curl** (for API testing)

### Installation & Setup

1. **Clone and navigate to the project:**
   ```bash
   git clone <repository-url>
   cd scheduler
   ```

2. **Build the project:**
   ```bash
   cargo build
   ```

3. **Run tests to verify everything works:**
   ```bash
   cargo test
   ```

## 🎯 Core Functionality Demo

### 1. Start the Application

```bash
cargo run
```

**Expected Output:**
```
INFO scheduler::logging: Logging system initialized level=info file=None format=text
INFO scheduler: Starting Job Scheduler application
INFO scheduler::storage::sqlite: Connecting to SQLite database database_url=sqlite:scheduler.db
INFO scheduler::storage::sqlite: SQLite storage initialized successfully
INFO scheduler: Storage initialized
INFO scheduler: Worker configured with concurrency: 4
INFO scheduler::board: Creating board UI routes ui_path=/board api_prefix=/board/api auth_enabled=false
INFO scheduler: Web server configured
INFO scheduler: Starting server on 127.0.0.1:8080
INFO scheduler: Starting Apalis worker...
INFO scheduler: Starting web server...
```

The application will start on `http://127.0.0.1:8080`

### 2. Access the Web Board UI

Open your browser and navigate to:
```
http://127.0.0.1:8080/board
```

This provides a web interface for monitoring jobs and system status.

### 3. Test API Endpoints

#### Health Check
```bash
curl http://127.0.0.1:8080/api/health
```

**Expected Response:**
```json
{
  "status": "healthy",
  "timestamp": "2025-05-28T12:29:07.009473115Z"
}
```

#### List Jobs
```bash
curl http://127.0.0.1:8080/api/jobs
```

**Expected Response:**
```json
{
  "count": 0,
  "jobs": []
}
```

#### Create SSH Job
```bash
curl -X POST http://127.0.0.1:8080/api/jobs \
  -H "Content-Type: application/json" \
  -d @test_job.json
```

**Expected Response:**
```json
{
  "id": "6133c748-2d29-4cc6-9994-42f8a26ce609",
  "status": "Pending",
  "message": "Job created successfully"
}
```

#### List Connection Profiles
```bash
curl http://127.0.0.1:8080/api/connections
```

**Expected Response:**
```json
{
  "count": 0,
  "profiles": []
}
```

### 4. Run Example Programs

#### Basic Usage Example
```bash
cargo run --example basic_usage
```

This demonstrates:
- Creating SSH job payloads
- Job validation
- Storage operations

#### Logging Demo
```bash
cargo run --example logging_demo
```

This demonstrates:
- Different logging configurations (text/JSON)
- Console and file logging
- Structured logging with job metadata

#### Board UI Example
```bash
cargo run --example with_board
```

This starts a minimal server with just the board UI for testing.

## 📊 API Reference

### Job Management

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/health` | System health check |
| POST | `/api/jobs` | Create a new SSH job |
| GET | `/api/jobs` | List all jobs |
| GET | `/api/jobs/{id}` | Get specific job details |
| DELETE | `/api/jobs/{id}` | Delete a job |
| GET | `/api/jobs/{id}/logs` | Get job execution logs |

### Connection Profiles

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/connections` | Create connection profile |
| GET | `/api/connections` | List connection profiles |
| GET | `/api/connections/{id}` | Get connection profile |
| PUT | `/api/connections/{id}` | Update connection profile |
| DELETE | `/api/connections/{id}` | Delete connection profile |

### Board UI

| Endpoint | Description |
|----------|-------------|
| GET | `/board` | Web dashboard for job monitoring |
| GET | `/board/api/*` | Board API endpoints |

## 🔧 Configuration

The application uses `config.toml` for configuration:

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
# file = "scheduler.log"  # Uncomment to enable file logging
# format = "json"  # Use "json" for structured logging
```

## 🧪 Testing

### Run All Tests
```bash
cargo test
```

### Run Specific Test Categories
```bash
# Run only logging tests
cargo test logging

# Run with verbose output
cargo test -- --nocapture
```

## 📝 Sample Job Payload

The `test_job.json` file contains a sample SSH job:

```json
{
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
    "description": "Test SSH job for device information gathering"
  }
}
```

## ✅ Verified Working Features

- ✅ **Application Startup**: Server starts successfully with all components
- ✅ **Logging System**: Structured logging with configurable levels and formats
- ✅ **SQLite Storage**: Database initialization and health checks
- ✅ **API Endpoints**: All REST endpoints respond correctly
- ✅ **Job Validation**: SSH job payload validation works
- ✅ **Board UI**: Web interface is accessible and functional
- ✅ **Configuration**: Environment-based configuration loading
- ✅ **Health Monitoring**: System health checks and status reporting
- ✅ **Error Handling**: Proper error responses and logging

## 🔄 Development Workflow

1. **Make changes** to the code
2. **Run tests** to verify functionality:
   ```bash
   cargo test
   ```
3. **Test examples** to verify integration:
   ```bash
   cargo run --example basic_usage
   ```
4. **Start the server** and test APIs:
   ```bash
   cargo run
   ```
5. **Check logs** for any issues or debugging information

## 🐛 Troubleshooting

### Common Issues

1. **Port already in use**: Change the port in `config.toml`
2. **Database connection issues**: Ensure write permissions in the project directory
3. **Build errors**: Run `cargo clean` and `cargo build` again

### Debug Mode

Run with debug logging:
```bash
RUST_LOG=debug cargo run
```

### Verbose Testing

Run tests with output:
```bash
cargo test -- --nocapture
```

## 🎯 Next Steps

The application provides a solid foundation for job scheduling. Current implementation includes:

- Complete API framework
- Job validation and storage abstraction
- Logging and monitoring
- Web UI framework
- Configuration management

Future enhancements can include:
- Actual SSH job execution
- Job persistence and queuing
- Advanced scheduling features
- Authentication and authorization
- Metrics and monitoring dashboards
