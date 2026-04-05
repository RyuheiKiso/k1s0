-- quota_usage テーブルの以下を修正する:
-- 1. RLS ポリシーに ::TEXT キャストを追加し、マルチテナント境界を確実にする
-- 2. tenant_id の DEFAULT '' を DEFAULT 'system' に変更する
--    （空文字列のままでは current_tenant_id='system' 時にアクセス不能となる）
-- 3. 既存の空文字列レコードを 'system' テナントへ移行する
BEGIN;

DROP POLICY IF EXISTS tenant_isolation ON quota.quota_usage;

ALTER TABLE quota.quota_usage ALTER COLUMN tenant_id SET DEFAULT 'system';

UPDATE quota.quota_usage SET tenant_id = 'system' WHERE tenant_id = '';

CREATE POLICY tenant_isolation ON quota.quota_usage
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

COMMIT;
