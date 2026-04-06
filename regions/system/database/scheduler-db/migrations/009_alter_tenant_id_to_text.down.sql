-- scheduler_jobs / job_executions の tenant_id を TEXT から VARCHAR(255) に戻す。

BEGIN;

SET LOCAL search_path TO scheduler, public;

DROP POLICY IF EXISTS tenant_isolation ON scheduler.scheduler_jobs;
DROP POLICY IF EXISTS tenant_isolation ON scheduler.job_executions;

-- テナントスコープ UNIQUE INDEX を削除し、元の name UNIQUE 制約を復元する
DROP INDEX IF EXISTS scheduler.uq_scheduler_jobs_tenant_name;
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'scheduler_jobs_name_key' AND conrelid = 'scheduler.scheduler_jobs'::regclass
    ) THEN
        ALTER TABLE scheduler.scheduler_jobs ADD CONSTRAINT scheduler_jobs_name_key UNIQUE (name);
    END IF;
END $$;

ALTER TABLE scheduler.scheduler_jobs
    ALTER COLUMN tenant_id TYPE VARCHAR(255) USING tenant_id::VARCHAR(255);

ALTER TABLE scheduler.job_executions
    ALTER COLUMN tenant_id TYPE VARCHAR(255) USING tenant_id::VARCHAR(255);

-- 008 マイグレーション相当のポリシーを復元する
CREATE POLICY tenant_isolation ON scheduler.scheduler_jobs
    AS RESTRICTIVE
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT)
    WITH CHECK (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

CREATE POLICY tenant_isolation ON scheduler.job_executions
    AS RESTRICTIVE
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT)
    WITH CHECK (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

COMMIT;
