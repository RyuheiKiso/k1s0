-- 005_fix_user_sessions_rls_cast のロールバック。
-- ::TEXT キャストなしの旧ポリシーを復元する。
BEGIN;

DROP POLICY IF EXISTS tenant_isolation ON session.user_sessions;
CREATE POLICY tenant_isolation ON session.user_sessions
    USING (tenant_id = current_setting('app.current_tenant_id', true));

COMMIT;
