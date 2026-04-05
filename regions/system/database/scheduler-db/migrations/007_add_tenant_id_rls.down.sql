-- 007_add_tenant_id_rls.up.sql のロールバック: RLS ポリシーと tenant_id カラムを削除する

BEGIN;

-- scheduler_jobs の RLS を無効化する
ALTER TABLE scheduler.scheduler_jobs DISABLE ROW LEVEL SECURITY;
ALTER TABLE scheduler.scheduler_jobs NO FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS tenant_isolation ON scheduler.scheduler_jobs;

-- job_executions の RLS を無効化する
ALTER TABLE scheduler.job_executions DISABLE ROW LEVEL SECURITY;
ALTER TABLE scheduler.job_executions NO FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS tenant_isolation ON scheduler.job_executions;

-- インデックスを削除する
DROP INDEX IF EXISTS scheduler.idx_scheduler_jobs_tenant_id;
DROP INDEX IF EXISTS scheduler.idx_scheduler_jobs_tenant_enabled;
DROP INDEX IF EXISTS scheduler.idx_job_executions_tenant_id;

-- tenant_id カラムを削除する
ALTER TABLE scheduler.scheduler_jobs DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE scheduler.job_executions DROP COLUMN IF EXISTS tenant_id;

COMMIT;
