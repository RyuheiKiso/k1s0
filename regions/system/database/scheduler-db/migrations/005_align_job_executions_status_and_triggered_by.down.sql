ALTER TABLE scheduler.job_executions
    DROP CONSTRAINT IF EXISTS chk_job_executions_status;

UPDATE scheduler.job_executions
SET status = 'completed'
WHERE status = 'succeeded';

ALTER TABLE scheduler.job_executions
    ADD CONSTRAINT chk_job_executions_status
    CHECK (status IN ('running', 'completed', 'failed'));

ALTER TABLE scheduler.job_executions
    DROP COLUMN IF EXISTS triggered_by;
