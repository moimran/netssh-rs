-- Add next_run_at field for recurring job scheduling
-- This field tracks when a recurring job should execute next

ALTER TABLE jobs ADD COLUMN next_run_at DATETIME;

-- Create index for efficient scheduling queries
CREATE INDEX IF NOT EXISTS idx_jobs_next_run_at ON jobs(next_run_at);

-- Create index for scheduled jobs queries
CREATE INDEX IF NOT EXISTS idx_jobs_cron_expression ON jobs(cron_expression);

-- Create composite index for scheduler queries
CREATE INDEX IF NOT EXISTS idx_jobs_scheduler_query ON jobs(status, next_run_at, scheduled_for);
