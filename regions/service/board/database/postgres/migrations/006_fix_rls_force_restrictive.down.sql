-- board_service の RLS ポリシーを 003 マイグレーション相当に戻す。

BEGIN;

SET LOCAL search_path TO board_service, public;

-- board_columns のポリシーを 003 相当に戻す（AS RESTRICTIVE / WITH CHECK なし）
DROP POLICY IF EXISTS tenant_isolation ON board_service.board_columns;
CREATE POLICY tenant_isolation ON board_service.board_columns
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);

-- FORCE を解除する
ALTER TABLE board_service.board_columns NO FORCE ROW LEVEL SECURITY;

COMMIT;
