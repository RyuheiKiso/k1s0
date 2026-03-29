-- 004_add_user_sessions_rls のロールバック
BEGIN;
DROP POLICY IF EXISTS tenant_isolation ON session.user_sessions;
ALTER TABLE session.user_sessions DISABLE ROW LEVEL SECURITY;
ALTER TABLE session.user_sessions NO FORCE ROW LEVEL SECURITY;
COMMIT;
