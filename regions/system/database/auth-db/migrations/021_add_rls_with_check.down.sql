-- WITH CHECK 付き RLS ポリシーを USING のみのポリシーに戻す（ロールバック用）
BEGIN;

-- api_keys テーブルの tenant_isolation ポリシーを USING のみで再作成する
DROP POLICY IF EXISTS tenant_isolation ON auth.api_keys;
CREATE POLICY tenant_isolation ON auth.api_keys
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

COMMIT;
