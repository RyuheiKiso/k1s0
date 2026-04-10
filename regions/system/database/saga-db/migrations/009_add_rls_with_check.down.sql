-- WITH CHECK 付き RLS ポリシーを USING のみのポリシーに戻す（ロールバック用）
BEGIN;

-- saga_states テーブルの tenant_isolation ポリシーを USING のみで再作成する
DROP POLICY IF EXISTS tenant_isolation ON saga.saga_states;
CREATE POLICY tenant_isolation ON saga.saga_states
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

-- saga_step_logs テーブルの tenant_isolation ポリシーを USING のみで再作成する
DROP POLICY IF EXISTS tenant_isolation ON saga.saga_step_logs;
CREATE POLICY tenant_isolation ON saga.saga_step_logs
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

COMMIT;
