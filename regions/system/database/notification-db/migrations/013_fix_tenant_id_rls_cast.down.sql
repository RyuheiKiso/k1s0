-- 013_fix_tenant_id_rls_cast のロールバック。
-- ::TEXT キャストなしの旧ポリシーを復元する。
BEGIN;

DROP POLICY IF EXISTS tenant_isolation ON notification.channels;
CREATE POLICY tenant_isolation ON notification.channels
    USING (tenant_id = current_setting('app.current_tenant_id', true));

COMMIT;
