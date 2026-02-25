ALTER TABLE scheduler.scheduler_jobs
    DROP COLUMN IF EXISTS description,
    DROP COLUMN IF EXISTS timezone,
    DROP COLUMN IF EXISTS target_type,
    DROP COLUMN IF EXISTS target;
