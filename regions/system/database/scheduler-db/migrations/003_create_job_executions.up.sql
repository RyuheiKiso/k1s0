CREATE TABLE IF NOT EXISTS scheduler.job_executions (
    id            UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    job_id        UUID         NOT NULL REFERENCES scheduler.scheduler_jobs(id) ON DELETE CASCADE,
    status        VARCHAR(50)  NOT NULL DEFAULT 'running',
    started_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    completed_at  TIMESTAMPTZ,
    error_message TEXT,

    CONSTRAINT chk_job_executions_status CHECK (status IN ('running', 'completed', 'failed'))
);

CREATE INDEX IF NOT EXISTS idx_job_executions_job_id ON scheduler.job_executions (job_id);
CREATE INDEX IF NOT EXISTS idx_job_executions_status ON scheduler.job_executions (status);
CREATE INDEX IF NOT EXISTS idx_job_executions_started_at ON scheduler.job_executions (started_at DESC);
