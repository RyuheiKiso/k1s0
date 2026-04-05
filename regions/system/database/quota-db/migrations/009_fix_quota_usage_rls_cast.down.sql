-- 009_fix_quota_usage_rls_cast のロールバック。
-- tenant_id の DEFAULT を空文字列に戻し、::TEXT キャストなしの旧ポリシーを復元する。
-- 注意: 'system' に更新されたレコードは元の空文字列に戻さない（データ整合性を優先）。
BEGIN;

DROP POLICY IF EXISTS tenant_isolation ON quota.quota_usage;

ALTER TABLE quota.quota_usage ALTER COLUMN tenant_id SET DEFAULT '';

CREATE POLICY tenant_isolation ON quota.quota_usage
    USING (tenant_id = current_setting('app.current_tenant_id', true));

COMMIT;
