ALTER TABLE scheduler.job_executions
    ADD COLUMN IF NOT EXISTS triggered_by VARCHAR(50) NOT NULL DEFAULT 'scheduler';

UPDATE scheduler.job_executions
SET status = 'succeeded'
WHERE status = 'completed';

ALTER TABLE scheduler.job_executions
    DROP CONSTRAINT IF EXISTS chk_job_executions_status;

ALTER TABLE scheduler.job_executions
    ADD CONSTRAINT chk_job_executions_status
    CHECK (status IN ('running', 'succeeded', 'failed'));
