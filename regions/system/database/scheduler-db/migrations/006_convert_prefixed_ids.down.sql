ALTER TABLE scheduler.job_executions
    DROP CONSTRAINT IF EXISTS job_executions_job_id_fkey;

ALTER TABLE scheduler.job_executions
    ALTER COLUMN id TYPE UUID
    USING regexp_replace(substr(id, 6), '(.{8})(.{4})(.{4})(.{4})(.{12})', '\\1-\\2-\\3-\\4-\\5')::uuid,
    ALTER COLUMN id SET DEFAULT gen_random_uuid(),
    ALTER COLUMN job_id TYPE UUID
    USING regexp_replace(substr(job_id, 5), '(.{8})(.{4})(.{4})(.{4})(.{12})', '\\1-\\2-\\3-\\4-\\5')::uuid;

ALTER TABLE scheduler.scheduler_jobs
    ALTER COLUMN id TYPE UUID
    USING regexp_replace(substr(id, 5), '(.{8})(.{4})(.{4})(.{4})(.{12})', '\\1-\\2-\\3-\\4-\\5')::uuid,
    ALTER COLUMN id SET DEFAULT gen_random_uuid();

ALTER TABLE scheduler.job_executions
    ADD CONSTRAINT job_executions_job_id_fkey
    FOREIGN KEY (job_id) REFERENCES scheduler.scheduler_jobs(id) ON DELETE CASCADE;
