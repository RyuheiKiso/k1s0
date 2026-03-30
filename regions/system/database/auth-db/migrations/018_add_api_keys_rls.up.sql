-- H-010 監査対応: auth.api_keys テーブルにマルチテナント用の行レベルセキュリティを追加する
-- テナント間のAPIキーデータ漏洩を DB 層で防止するため RLS を有効化する
-- パターン: saga-db の 008_add_tenant_id_rls.up.sql に準拠

BEGIN;

-- api_keys テーブルの RLS を有効化する
ALTER TABLE auth.api_keys ENABLE ROW LEVEL SECURITY;
-- スーパーユーザー・テーブルオーナーもポリシーに従わせる（バイパス防止）
ALTER TABLE auth.api_keys FORCE ROW LEVEL SECURITY;

-- テナント分離ポリシー: 現在のセッション変数と一致するテナントのデータのみアクセス可能
CREATE POLICY tenant_isolation ON auth.api_keys
    USING (tenant_id = current_setting('app.current_tenant_id', true));

COMMIT;
