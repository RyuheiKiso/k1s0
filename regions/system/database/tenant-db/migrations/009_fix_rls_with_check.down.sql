-- tenant テーブルのポリシーを 008 マイグレーション相当に戻す（WITH CHECK / AS RESTRICTIVE なし）。

BEGIN;

SET LOCAL search_path TO tenant, public;

-- tenants テーブルのポリシーを 008 相当に戻す（WITH CHECK なし）
DROP POLICY IF EXISTS tenant_isolation_policy ON tenant.tenants;
CREATE POLICY tenant_isolation_policy ON tenant.tenants
    USING (id::TEXT = current_setting('app.current_tenant_id', true));

-- tenant_members テーブルのポリシーを 008 相当に戻す（WITH CHECK なし）
DROP POLICY IF EXISTS tenant_member_isolation_policy ON tenant.tenant_members;
CREATE POLICY tenant_member_isolation_policy ON tenant.tenant_members
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true));

COMMIT;
