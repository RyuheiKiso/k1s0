-- H-010 監査対応: session.user_sessions テーブルにマルチテナント用の行レベルセキュリティを追加する
-- テナント間のセッションデータ漏洩を DB 層で防止するため RLS を有効化する

BEGIN;

ALTER TABLE session.user_sessions ENABLE ROW LEVEL SECURITY;
ALTER TABLE session.user_sessions FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON session.user_sessions
    USING (tenant_id = current_setting('app.current_tenant_id', true));

COMMIT;
