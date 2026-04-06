-- activity_service の RLS ポリシーを 003 マイグレーション相当に戻す。

BEGIN;

SET LOCAL search_path TO activity_service, public;

-- activities のポリシーを 003 相当に戻す
DROP POLICY IF EXISTS tenant_isolation ON activity_service.activities;
CREATE POLICY tenant_isolation ON activity_service.activities
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);
ALTER TABLE activity_service.activities NO FORCE ROW LEVEL SECURITY;

COMMIT;
