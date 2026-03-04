ALTER TABLE scheduler.job_executions
    DROP CONSTRAINT IF EXISTS job_executions_job_id_fkey;

ALTER TABLE scheduler.scheduler_jobs
    ALTER COLUMN id DROP DEFAULT,
    ALTER COLUMN id TYPE VARCHAR(64)
    USING ('job_' || replace(id::text, '-', ''));

ALTER TABLE scheduler.job_executions
    ALTER COLUMN id DROP DEFAULT,
    ALTER COLUMN id TYPE VARCHAR(64)
    USING ('exec_' || replace(id::text, '-', '')),
    ALTER COLUMN job_id TYPE VARCHAR(64)
    USING ('job_' || replace(job_id::text, '-', ''));

ALTER TABLE scheduler.job_executions
    ADD CONSTRAINT job_executions_job_id_fkey
    FOREIGN KEY (job_id) REFERENCES scheduler.scheduler_jobs(id) ON DELETE CASCADE;
