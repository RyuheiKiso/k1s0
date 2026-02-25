CREATE TABLE IF NOT EXISTS scheduler.scheduler_jobs (
    id              UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(255) NOT NULL UNIQUE,
    cron_expression VARCHAR(255) NOT NULL,
    job_type        VARCHAR(50)  NOT NULL DEFAULT 'default',
    payload         JSONB        NOT NULL DEFAULT '{}',
    enabled         BOOLEAN      NOT NULL DEFAULT true,
    max_retries     INT          NOT NULL DEFAULT 3,
    last_run_at     TIMESTAMPTZ,
    next_run_at     TIMESTAMPTZ,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_scheduler_jobs_name ON scheduler.scheduler_jobs (name);
CREATE INDEX IF NOT EXISTS idx_scheduler_jobs_enabled ON scheduler.scheduler_jobs (enabled);
CREATE INDEX IF NOT EXISTS idx_scheduler_jobs_next_run_at ON scheduler.scheduler_jobs (next_run_at);

CREATE TRIGGER trigger_scheduler_jobs_update_updated_at
    BEFORE UPDATE ON scheduler.scheduler_jobs
    FOR EACH ROW
    EXECUTE FUNCTION scheduler.update_updated_at();
