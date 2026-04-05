-- user_sessions テーブルの RLS ポリシーに ::TEXT キャストを追加し、
-- tenant_id 型と current_setting() の戻り値型を明示的に統一する。
-- これにより、マルチテナント境界が確実に機能する。
BEGIN;

DROP POLICY IF EXISTS tenant_isolation ON session.user_sessions;
CREATE POLICY tenant_isolation ON session.user_sessions
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

COMMIT;
