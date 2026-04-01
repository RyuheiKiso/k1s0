-- テナント分離のために追加した Row Level Security ポリシーを削除し、RLS を無効化する
-- 008_add_rls_policies.up.sql の逆操作を逆順で実行する
-- sqlx migrate revert で安全にロールバックできるようにする（ADR-0054準拠）

BEGIN;

-- サービスアカウントのセッション変数設定権限を取り消す
-- GRANT SET ON PARAMETER の逆操作（PostgreSQL 15 以降）
REVOKE SET ON PARAMETER app.current_tenant_id FROM PUBLIC;

-- tenant_members テーブルの RLS を無効化する
-- FORCE → DROP POLICY → DISABLE の順で解除する
ALTER TABLE tenant.tenant_members NO FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS tenant_member_isolation_policy ON tenant.tenant_members;
ALTER TABLE tenant.tenant_members DISABLE ROW LEVEL SECURITY;

-- tenants テーブルの RLS を無効化する
-- FORCE → DROP POLICY → DISABLE の順で解除する
ALTER TABLE tenant.tenants NO FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS tenant_isolation_policy ON tenant.tenants;
ALTER TABLE tenant.tenants DISABLE ROW LEVEL SECURITY;

COMMIT;
