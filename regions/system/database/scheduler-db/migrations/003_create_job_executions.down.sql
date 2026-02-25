DROP INDEX IF EXISTS scheduler.idx_job_executions_started_at;
DROP INDEX IF EXISTS scheduler.idx_job_executions_status;
DROP INDEX IF EXISTS scheduler.idx_job_executions_job_id;
DROP TABLE IF EXISTS scheduler.job_executions;
