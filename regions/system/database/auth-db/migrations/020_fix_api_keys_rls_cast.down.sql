-- 020_fix_api_keys_rls_cast のロールバック。
-- ::TEXT キャストなしの旧ポリシーを復元する。
BEGIN;

DROP POLICY IF EXISTS tenant_isolation ON auth.api_keys;
-- 旧ポリシーを復元（キャストなし）
CREATE POLICY tenant_isolation ON auth.api_keys
    USING (tenant_id = current_setting('app.current_tenant_id', true));

COMMIT;
