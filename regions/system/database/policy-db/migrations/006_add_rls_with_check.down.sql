-- WITH CHECK 付き RLS ポリシーを USING のみのポリシーに戻す（ロールバック用）
BEGIN;

-- policies テーブルの tenant_isolation ポリシーを USING のみで再作成する
DROP POLICY IF EXISTS tenant_isolation ON policy.policies;
CREATE POLICY tenant_isolation ON policy.policies
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

-- policy_bundles テーブルの tenant_isolation ポリシーを USING のみで再作成する
DROP POLICY IF EXISTS tenant_isolation ON policy.policy_bundles;
CREATE POLICY tenant_isolation ON policy.policy_bundles
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

COMMIT;
