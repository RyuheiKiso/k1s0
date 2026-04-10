-- WITH CHECK 付き RLS ポリシーを USING のみのポリシーに戻す（ロールバック用）
BEGIN;

-- scheduler_jobs テーブルの tenant_isolation ポリシーを USING のみで再作成する
DROP POLICY IF EXISTS tenant_isolation ON scheduler.scheduler_jobs;
CREATE POLICY tenant_isolation ON scheduler.scheduler_jobs
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

-- job_executions テーブルの tenant_isolation ポリシーを USING のみで再作成する
DROP POLICY IF EXISTS tenant_isolation ON scheduler.job_executions;
CREATE POLICY tenant_isolation ON scheduler.job_executions
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

COMMIT;
