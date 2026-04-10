-- RLS ポリシーに WITH CHECK 句を追加して INSERT/UPDATE 時のテナント検証を有効化する
-- USING 句のみの場合、SELECT/UPDATE/DELETE はテナント分離されるが INSERT は検証されない（CWE-284）
-- AS RESTRICTIVE により他のポリシーが存在しても必ずこのポリシーで制限される
BEGIN;

-- user_sessions テーブルの tenant_isolation ポリシーを WITH CHECK 付きで再作成する
DROP POLICY IF EXISTS tenant_isolation ON session.user_sessions;
CREATE POLICY tenant_isolation ON session.user_sessions
    AS RESTRICTIVE
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT)
    WITH CHECK (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

COMMIT;
