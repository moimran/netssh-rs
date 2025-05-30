-- Initial database schema for job scheduler
-- Jobs table to store job metadata
CREATE TABLE IF NOT EXISTS jobs (
    id TEXT PRIMARY KEY,
    job_type TEXT NOT NULL,
    payload TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    started_at DATETIME,
    completed_at DATETIME,
    scheduled_for DATETIME,
    cron_expression TEXT,
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    error_message TEXT,
    worker_id TEXT
);

-- Job results table to store command execution results
CREATE TABLE IF NOT EXISTS job_results (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL,
    command TEXT NOT NULL,
    output TEXT,
    error TEXT,
    exit_code INTEGER,
    executed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    duration_ms INTEGER,
    FOREIGN KEY (job_id) REFERENCES jobs (id) ON DELETE CASCADE
);

-- Job logs table for detailed logging
CREATE TABLE IF NOT EXISTS job_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id TEXT NOT NULL,
    level TEXT NOT NULL,
    message TEXT NOT NULL,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    context TEXT,
    FOREIGN KEY (job_id) REFERENCES jobs (id) ON DELETE CASCADE
);

-- SSH connections table for reusable connection profiles
CREATE TABLE IF NOT EXISTS ssh_connections (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    host TEXT NOT NULL,
    username TEXT NOT NULL,
    device_type TEXT NOT NULL,
    port INTEGER DEFAULT 22,
    timeout_seconds INTEGER DEFAULT 60,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for better performance
CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status);
CREATE INDEX IF NOT EXISTS idx_jobs_created_at ON jobs(created_at);
CREATE INDEX IF NOT EXISTS idx_jobs_scheduled_for ON jobs(scheduled_for);
CREATE INDEX IF NOT EXISTS idx_jobs_job_type ON jobs(job_type);
CREATE INDEX IF NOT EXISTS idx_job_results_job_id ON job_results(job_id);
CREATE INDEX IF NOT EXISTS idx_job_results_executed_at ON job_results(executed_at);
CREATE INDEX IF NOT EXISTS idx_job_logs_job_id ON job_logs(job_id);
CREATE INDEX IF NOT EXISTS idx_job_logs_timestamp ON job_logs(timestamp);
CREATE INDEX IF NOT EXISTS idx_ssh_connections_name ON ssh_connections(name);
