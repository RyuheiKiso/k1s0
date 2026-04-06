-- tenant テーブルの RLS ポリシーに WITH CHECK を追加する。
-- HIGH-DB-004 監査対応: 008 マイグレーションで USING 句は設定されたが WITH CHECK が欠落していた。
-- WITH CHECK がないと INSERT / UPDATE 時のテナント検証が行われない（CWE-284）。
-- AS RESTRICTIVE も追加して他のポリシーと AND 結合されるように設定する。
-- 既存ポリシーを DROP してから完全なポリシーで再作成する。

BEGIN;

SET LOCAL search_path TO tenant, public;

-- tenants テーブルのポリシーを AS RESTRICTIVE + WITH CHECK 付きで再作成する
DROP POLICY IF EXISTS tenant_isolation_policy ON tenant.tenants;
CREATE POLICY tenant_isolation_policy ON tenant.tenants
    AS RESTRICTIVE
    USING (id::TEXT = current_setting('app.current_tenant_id', true))
    WITH CHECK (id::TEXT = current_setting('app.current_tenant_id', true));

-- tenant_members テーブルのポリシーを AS RESTRICTIVE + WITH CHECK 付きで再作成する
DROP POLICY IF EXISTS tenant_member_isolation_policy ON tenant.tenant_members;
CREATE POLICY tenant_member_isolation_policy ON tenant.tenant_members
    AS RESTRICTIVE
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id::TEXT = current_setting('app.current_tenant_id', true));

COMMIT;
