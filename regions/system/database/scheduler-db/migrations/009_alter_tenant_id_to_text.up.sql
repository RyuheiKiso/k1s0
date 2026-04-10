-- scheduler_jobs / job_executions テーブルの tenant_id を VARCHAR(255) から TEXT 型に変更する。
-- CRITICAL-DB-002 監査対応: 全サービスで tenant_id を TEXT 型に統一する（ADR-0093）。
-- HIGH-DB-007: UNIQUE(name) を UNIQUE(tenant_id, name) に変更してテナント間重複を許可する。
-- 既存の RLS ポリシーを DROP してから型変更し、AS RESTRICTIVE + WITH CHECK で再作成する。

BEGIN;

SET LOCAL search_path TO scheduler, public;

-- 既存の RLS ポリシーを先に削除する
DROP POLICY IF EXISTS tenant_isolation ON scheduler.scheduler_jobs;
DROP POLICY IF EXISTS tenant_isolation ON scheduler.job_executions;

-- scheduler_jobs テーブルの tenant_id を TEXT 型に変更する
ALTER TABLE scheduler.scheduler_jobs
    ALTER COLUMN tenant_id TYPE TEXT USING tenant_id::TEXT;

-- job_executions テーブルの tenant_id を TEXT 型に変更する
ALTER TABLE scheduler.job_executions
    ALTER COLUMN tenant_id TYPE TEXT USING tenant_id::TEXT;

-- scheduler_jobs テーブルの UNIQUE(name) を UNIQUE(tenant_id, name) に変更する（HIGH-DB-007 対応）
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'scheduler_jobs_name_key' AND conrelid = 'scheduler.scheduler_jobs'::regclass
    ) THEN
        ALTER TABLE scheduler.scheduler_jobs DROP CONSTRAINT scheduler_jobs_name_key;
    END IF;
END $$;
CREATE UNIQUE INDEX IF NOT EXISTS uq_scheduler_jobs_tenant_name
    ON scheduler.scheduler_jobs (tenant_id, name);

-- scheduler_jobs テーブルのポリシーを再作成する
CREATE POLICY tenant_isolation ON scheduler.scheduler_jobs
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- job_executions テーブルのポリシーを再作成する
CREATE POLICY tenant_isolation ON scheduler.job_executions
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

COMMIT;
