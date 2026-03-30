-- H-010 監査対応: quota.quota_usage テーブルにマルチテナント用の行レベルセキュリティを追加する
-- テナント間のクォータデータ漏洩を DB 層で防止するため RLS を有効化する

BEGIN;

ALTER TABLE quota.quota_usage ENABLE ROW LEVEL SECURITY;
ALTER TABLE quota.quota_usage FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON quota.quota_usage
    USING (tenant_id = current_setting('app.current_tenant_id', true));

COMMIT;
