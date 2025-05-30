# Job Scheduler Architecture Documentation

## Table of Contents
1. [System Architecture Overview](#system-architecture-overview)
2. [User Flow Diagrams](#user-flow-diagrams)
3. [Code Flow Diagrams](#code-flow-diagrams)
4. [Component Interaction Diagrams](#component-interaction-diagrams)
5. [Database Schema](#database-schema)
6. [Deployment Architecture](#deployment-architecture)

## System Architecture Overview

The Job Scheduler is a distributed system built with Rust, featuring a modular architecture that separates concerns across multiple layers.

```mermaid
graph TB
    subgraph "Client Layer"
        UI[Web Dashboard]
        API_CLIENT[API Clients]
        CLI[CLI Tools]
    end
    
    subgraph "Application Layer"
        WEB[Web Server<br/>Axum + Tower]
        API[REST API Layer<br/>Job Management]
        BOARD[Board Service<br/>Web UI]
    end
    
    subgraph "Business Logic Layer"
        WORKER[Apalis Workers<br/>Job Processing]
        SSH[SSH Job Handler<br/>netssh-core]
        VALIDATION[Job Validation<br/>Business Rules]
    end
    
    subgraph "Infrastructure Layer"
        QUEUE[(Apalis Queue<br/>SQLite)]
        STORAGE[(Application Storage<br/>SQLite)]
        CONFIG[Configuration<br/>Environment/TOML]
        LOGGING[Structured Logging<br/>Tracing]
    end
    
    subgraph "External Systems"
        DEVICES[Network Devices<br/>SSH Targets]
    end
    
    UI --> WEB
    API_CLIENT --> API
    CLI --> API
    
    WEB --> API
    WEB --> BOARD
    
    API --> WORKER
    API --> STORAGE
    BOARD --> STORAGE
    
    WORKER --> SSH
    WORKER --> VALIDATION
    WORKER --> QUEUE
    WORKER --> STORAGE
    
    SSH --> DEVICES
    
    API --> CONFIG
    WORKER --> CONFIG
    WEB --> CONFIG
    
    API --> LOGGING
    WORKER --> LOGGING
    SSH --> LOGGING
    
    style UI fill:#e1f5fe
    style API_CLIENT fill:#e1f5fe
    style CLI fill:#e1f5fe
    style WEB fill:#f3e5f5
    style API fill:#f3e5f5
    style BOARD fill:#f3e5f5
    style WORKER fill:#e8f5e8
    style SSH fill:#e8f5e8
    style VALIDATION fill:#e8f5e8
    style QUEUE fill:#fff3e0
    style STORAGE fill:#fff3e0
    style CONFIG fill:#fff3e0
    style LOGGING fill:#fff3e0
    style DEVICES fill:#ffebee
```

### Key Components

- **Web Server**: Axum-based HTTP server with middleware for CORS, tracing, and routing
- **REST API**: RESTful endpoints for job management, health checks, and connection profiles
- **Apalis Workers**: Background job processing with configurable concurrency
- **SSH Handler**: Network device command execution using netssh-core library
- **Storage Layer**: Abstracted storage with SQLite implementation
- **Web Dashboard**: Real-time job monitoring and management interface

## User Flow Diagrams

### SSH Job Creation and Execution Flow

```mermaid
sequenceDiagram
    participant User
    participant WebUI as Web Dashboard
    participant API as REST API
    participant Storage as SQLite Storage
    participant Queue as Apalis Queue
    participant Worker as SSH Worker
    participant Device as Network Device
    
    User->>WebUI: Access Dashboard
    WebUI->>API: GET /api/jobs (load existing)
    API->>Storage: Query jobs
    Storage-->>API: Return job list
    API-->>WebUI: Job data
    WebUI-->>User: Display dashboard
    
    User->>API: POST /api/jobs (create SSH job)
    Note over API: Validate job payload
    API->>Storage: Save job record (pending)
    API->>Queue: Enqueue job for processing
    API-->>User: Return job ID
    
    Queue->>Worker: Dequeue job
    Worker->>Storage: Update status (running)
    Worker->>Device: SSH connect & authenticate
    Device-->>Worker: Connection established
    
    loop For each command
        Worker->>Device: Execute command
        Device-->>Worker: Command output
        Worker->>Storage: Save command result
    end
    
    Worker->>Device: Close SSH connection
    Worker->>Storage: Update job status (completed/failed)
    Worker->>Storage: Save final job result
    
    WebUI->>API: GET /api/jobs (auto-refresh)
    API->>Storage: Query updated jobs
    Storage-->>API: Return updated list
    API-->>WebUI: Updated job data
    WebUI-->>User: Show job completion

### Job Monitoring and Management Flow

```mermaid
flowchart TD
    START([User Opens Dashboard]) --> LOAD[Load Dashboard]
    LOAD --> FETCH[Fetch Jobs via API]
    FETCH --> DISPLAY[Display Job List & Stats]

    DISPLAY --> REFRESH{Auto Refresh<br/>Every 30s}
    REFRESH -->|Yes| FETCH

    DISPLAY --> MANUAL[Manual Refresh Button]
    MANUAL --> FETCH

    DISPLAY --> SELECT[Select Job for Details]
    SELECT --> DETAILS[GET /api/jobs/:id]
    DETAILS --> LOGS[GET /api/jobs/:id/logs]
    LOGS --> SHOW[Show Job Details & Logs]

    DISPLAY --> DELETE[Delete Job]
    DELETE --> CONFIRM{Confirm Deletion}
    CONFIRM -->|Yes| API_DELETE[DELETE /api/jobs/:id]
    CONFIRM -->|No| DISPLAY
    API_DELETE --> FETCH

    style START fill:#e1f5fe
    style LOAD fill:#f3e5f5
    style DISPLAY fill:#e8f5e8
    style REFRESH fill:#fff3e0
    style SHOW fill:#e8f5e8
```

## Code Flow Diagrams

### Request Processing Flow

```mermaid
flowchart TD
    REQUEST[HTTP Request] --> MIDDLEWARE[Tower Middleware Stack]
    MIDDLEWARE --> CORS[CORS Layer]
    CORS --> TRACE[Tracing Layer]
    TRACE --> ROUTER[Axum Router]

    ROUTER --> API_ROUTES{Route Type}
    API_ROUTES -->|/api/*| API_HANDLER[API Handlers]
    API_ROUTES -->|/board/*| BOARD_HANDLER[Board Service]
    API_ROUTES -->|/health| HEALTH[Health Check]

    API_HANDLER --> EXTRACT[Extract Request Data]
    EXTRACT --> VALIDATE[Validate Input]
    VALIDATE --> BUSINESS[Business Logic]

    BUSINESS --> STORAGE_OP[Storage Operations]
    BUSINESS --> QUEUE_OP[Queue Operations]

    STORAGE_OP --> DB[SQLite Database]
    QUEUE_OP --> APALIS[Apalis Queue]

    DB --> RESPONSE[Build Response]
    APALIS --> RESPONSE
    BOARD_HANDLER --> STATIC[Serve Static HTML]
    HEALTH --> HEALTH_CHECK[Database Health Check]

    RESPONSE --> JSON[JSON Response]
    STATIC --> HTML[HTML Response]
    HEALTH_CHECK --> STATUS[Status Response]

    JSON --> CLIENT[Client]
    HTML --> CLIENT
    STATUS --> CLIENT

    style REQUEST fill:#e1f5fe
    style MIDDLEWARE fill:#f3e5f5
    style API_HANDLER fill:#e8f5e8
    style STORAGE_OP fill:#fff3e0
    style CLIENT fill:#ffebee
```

### SSH Job Execution Workflow

```mermaid
flowchart TD
    DEQUEUE[Worker Dequeues Job] --> EXTRACT[Extract Job Payload]
    EXTRACT --> VALIDATE[Validate SSH Job]
    VALIDATE --> LOG_START[Log Job Start]
    LOG_START --> UPDATE_STATUS[Update Status: Running]

    UPDATE_STATUS --> SSH_CONNECT[Create SSH Connection]
    SSH_CONNECT --> AUTH{Authentication}
    AUTH -->|Success| ENABLE[Enable Privileged Mode]
    AUTH -->|Failure| ERROR_AUTH[Log Auth Error]

    ENABLE --> COMMANDS[Execute Commands Loop]
    ERROR_AUTH --> FAIL_JOB[Mark Job Failed]

    COMMANDS --> CMD_START[Start Command Execution]
    CMD_START --> CMD_EXEC[Execute Single Command]
    CMD_EXEC --> CMD_RESULT{Command Result}

    CMD_RESULT -->|Success| LOG_SUCCESS[Log Command Success]
    CMD_RESULT -->|Error| LOG_ERROR[Log Command Error]

    LOG_SUCCESS --> SAVE_RESULT[Save Command Result]
    LOG_ERROR --> SAVE_RESULT

    SAVE_RESULT --> MORE_CMDS{More Commands?}
    MORE_CMDS -->|Yes| CMD_START
    MORE_CMDS -->|No| CLOSE_SSH[Close SSH Connection]

    CLOSE_SSH --> COMPLETE[Mark Job Completed]
    FAIL_JOB --> SAVE_ERROR[Save Error Details]

    COMPLETE --> SAVE_FINAL[Save Final Job Result]
    SAVE_ERROR --> SAVE_FINAL

    SAVE_FINAL --> DONE[Job Processing Complete]

    style DEQUEUE fill:#e1f5fe
    style VALIDATE fill:#f3e5f5
    style SSH_CONNECT fill:#e8f5e8
    style COMMANDS fill:#fff3e0
    style DONE fill:#e8f5e8
```

### Error Handling and Retry Mechanism

```mermaid
flowchart TD
    ERROR[Job Execution Error] --> CHECK_RETRY{Retry Count < Max?}
    CHECK_RETRY -->|Yes| INCREMENT[Increment Retry Count]
    CHECK_RETRY -->|No| FINAL_FAIL[Mark Job Failed]

    INCREMENT --> DELAY[Apply Retry Delay]
    DELAY --> LOG_RETRY[Log Retry Attempt]
    LOG_RETRY --> REQUEUE[Requeue Job]
    REQUEUE --> WORKER[Worker Picks Up Job]

    WORKER --> RETRY_EXEC[Retry Execution]
    RETRY_EXEC --> SUCCESS{Execution Success?}
    SUCCESS -->|Yes| COMPLETE[Mark Completed]
    SUCCESS -->|No| ERROR

    FINAL_FAIL --> LOG_FINAL[Log Final Failure]
    LOG_FINAL --> NOTIFY[Notify Failure]

    COMPLETE --> LOG_SUCCESS[Log Success]
    LOG_SUCCESS --> CLEANUP[Cleanup Resources]

    style ERROR fill:#ffebee
    style CHECK_RETRY fill:#fff3e0
    style FINAL_FAIL fill:#ffcdd2
    style COMPLETE fill:#c8e6c9
```

## Component Interaction Diagrams

### API Layer to Storage Layer Interaction

```mermaid
sequenceDiagram
    participant API as API Handler
    participant Storage as Storage Trait
    participant SQLite as SQLite Implementation
    participant DB as SQLite Database

    API->>Storage: save_job_result(job_result)
    Storage->>SQLite: save_job_result(job_result)
    SQLite->>DB: INSERT/UPDATE jobs table
    DB-->>SQLite: Success/Error
    SQLite-->>Storage: Result
    Storage-->>API: Result

    API->>Storage: get_job_result(job_id)
    Storage->>SQLite: get_job_result(job_id)
    SQLite->>DB: SELECT from jobs WHERE id
    DB-->>SQLite: Job data
    SQLite-->>Storage: JobResult
    Storage-->>API: JobResult

    API->>Storage: save_command_results(job_id, results)
    Storage->>SQLite: save_command_results(job_id, results)
    SQLite->>DB: INSERT into job_results
    DB-->>SQLite: Success
    SQLite-->>Storage: Success
    Storage-->>API: Success
```

### Apalis Queue and Worker Interaction

```mermaid
sequenceDiagram
    participant API as API Handler
    participant Queue as Apalis Queue
    participant Worker as SSH Worker
    participant Storage as Storage Layer
    participant SSH as SSH Handler

    API->>Queue: push(ssh_job_payload)
    Queue->>Queue: Store job in queue table

    Worker->>Queue: poll for jobs
    Queue-->>Worker: ssh_job_payload

    Worker->>Storage: update_job_status(Running)
    Worker->>SSH: execute_ssh_commands(job)
    SSH-->>Worker: command_results

    Worker->>Storage: save_command_results(results)
    Worker->>Storage: save_job_result(final_result)

    Worker->>Queue: acknowledge job completion
    Queue->>Queue: Remove job from queue
```

### Configuration and Logging Integration

```mermaid
graph TD
    subgraph "Configuration Flow"
        ENV[Environment Variables] --> CONFIG[Config Builder]
        TOML[config.toml File] --> CONFIG
        CONFIG --> APP_CONFIG[Application Config]
    end

    subgraph "Application Components"
        MAIN[Main Application] --> WEB_SERVER[Web Server]
        MAIN --> WORKER[Apalis Worker]
        MAIN --> STORAGE[Storage Layer]
    end

    subgraph "Logging Flow"
        TRACING[Tracing Subscriber] --> CONSOLE[Console Output]
        TRACING --> FILE[Log Files]
        TRACING --> JSON[JSON Format]
    end

    APP_CONFIG --> MAIN
    APP_CONFIG --> WEB_SERVER
    APP_CONFIG --> WORKER
    APP_CONFIG --> STORAGE

    WEB_SERVER --> TRACING
    WORKER --> TRACING
    STORAGE --> TRACING

    style CONFIG fill:#e1f5fe
    style APP_CONFIG fill:#f3e5f5
    style TRACING fill:#e8f5e8
```

## Database Schema

### Entity Relationship Diagram

```mermaid
erDiagram
    JOBS {
        TEXT id PK
        TEXT job_type
        TEXT payload
        TEXT status
        DATETIME created_at
        DATETIME started_at
        DATETIME completed_at
        DATETIME scheduled_for
        TEXT cron_expression
        INTEGER retry_count
        INTEGER max_retries
        TEXT error_message
        TEXT worker_id
    }

    JOB_RESULTS {
        TEXT id PK
        TEXT job_id FK
        TEXT command
        TEXT output
        TEXT error
        INTEGER exit_code
        DATETIME executed_at
        INTEGER duration_ms
    }

    JOB_LOGS {
        INTEGER id PK
        TEXT job_id FK
        TEXT level
        TEXT message
        DATETIME timestamp
        TEXT context
    }

    SSH_CONNECTIONS {
        TEXT id PK
        TEXT name
        TEXT host
        TEXT username
        TEXT device_type
        INTEGER port
        INTEGER timeout_seconds
        DATETIME created_at
        DATETIME updated_at
    }

    JOBS ||--o{ JOB_RESULTS : "has many"
    JOBS ||--o{ JOB_LOGS : "has many"
```

### Database Schema Details

**Jobs Table**: Core job metadata and status tracking
- Primary key: UUID as TEXT
- Status values: pending, running, completed, failed, cancelled, retrying
- Supports both immediate and scheduled execution
- Tracks retry attempts and error messages

**Job Results Table**: Individual command execution results
- Links to parent job via foreign key
- Stores command text, output, errors, and timing
- Supports multiple commands per job

**Job Logs Table**: Detailed logging for debugging and monitoring
- Structured logging with levels (info, warn, error, debug)
- Contextual information for troubleshooting
- Timestamped entries for audit trails

**SSH Connections Table**: Reusable connection profiles
- Named connection configurations
- Device type classification for protocol handling
- Audit trail with creation and update timestamps

## Deployment Architecture

### Single Node Deployment

```mermaid
graph TB
    subgraph "Host Machine"
        subgraph "Job Scheduler Process"
            WEB[Web Server<br/>:8080]
            WORKER[Apalis Workers<br/>Configurable Concurrency]
            API[REST API<br/>Axum Framework]
            BOARD[Web Dashboard<br/>Static HTML]
        end

        subgraph "Storage"
            APP_DB[(Application DB<br/>scheduler.db)]
            QUEUE_DB[(Queue DB<br/>apalis_queue.db)]
        end

        subgraph "Configuration"
            CONFIG_FILE[config.toml]
            ENV_VARS[Environment Variables]
        end

        subgraph "Logging"
            CONSOLE[Console Output]
            LOG_FILES[Log Files<br/>Optional]
        end
    end

    subgraph "External Network"
        DEVICES[Network Devices<br/>SSH Targets]
        CLIENTS[API Clients<br/>Web Browsers]
    end

    WEB --> API
    WEB --> BOARD
    API --> APP_DB
    WORKER --> QUEUE_DB
    WORKER --> APP_DB
    WORKER --> DEVICES

    CONFIG_FILE --> WEB
    CONFIG_FILE --> WORKER
    ENV_VARS --> WEB
    ENV_VARS --> WORKER

    WEB --> CONSOLE
    WORKER --> CONSOLE
    WEB --> LOG_FILES
    WORKER --> LOG_FILES

    CLIENTS --> WEB

    style WEB fill:#e1f5fe
    style WORKER fill:#f3e5f5
    style APP_DB fill:#fff3e0
    style DEVICES fill:#ffebee
```

### Configuration Options

The application supports flexible configuration through multiple sources:

```toml
# config.toml example
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
file = "/var/log/scheduler.log"
format = "json"
rotation = "daily"
```

### Environment Variables

All configuration can be overridden using environment variables with the `SCHEDULER_` prefix:

- `SCHEDULER_DATABASE_URL`
- `SCHEDULER_SERVER_HOST`
- `SCHEDULER_SERVER_PORT`
- `SCHEDULER_WORKER_CONCURRENCY`
- `SCHEDULER_LOGGING_LEVEL`

### Process Management

The application runs as a single process with two main async tasks:
1. **Web Server**: Handles HTTP requests and serves the dashboard
2. **Worker Pool**: Processes jobs from the Apalis queue

Both tasks run concurrently using `tokio::select!` and can be gracefully shut down.

### Resource Requirements

**Minimum Requirements**:
- RAM: 64MB
- CPU: 1 core
- Disk: 100MB (plus log storage)
- Network: SSH access to target devices

**Recommended for Production**:
- RAM: 256MB
- CPU: 2+ cores
- Disk: 1GB+ (with log rotation)
- Network: Reliable connectivity to target devices

### Security Considerations

- **SSH Credentials**: Stored in database, consider encryption at rest
- **Network Access**: Requires SSH (port 22) access to target devices
- **Web Interface**: No authentication by default, enable auth for production
- **API Access**: No rate limiting implemented, consider reverse proxy
- **Database**: SQLite file permissions should be restricted

### Monitoring and Observability

- **Health Endpoint**: `/api/health` for load balancer checks
- **Structured Logging**: JSON format for log aggregation
- **Job Metrics**: Available through dashboard and API
- **Tracing**: Built-in request tracing for debugging

This architecture provides a solid foundation for network automation tasks while maintaining simplicity and reliability.
