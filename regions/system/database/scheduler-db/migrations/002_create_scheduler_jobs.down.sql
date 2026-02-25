DROP TRIGGER IF EXISTS trigger_scheduler_jobs_update_updated_at ON scheduler.scheduler_jobs;
DROP INDEX IF EXISTS scheduler.idx_scheduler_jobs_next_run_at;
DROP INDEX IF EXISTS scheduler.idx_scheduler_jobs_enabled;
DROP INDEX IF EXISTS scheduler.idx_scheduler_jobs_name;
DROP TABLE IF EXISTS scheduler.scheduler_jobs;
