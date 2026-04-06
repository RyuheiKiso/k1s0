-- activity_service の RLS ポリシーに FORCE + AS RESTRICTIVE + WITH CHECK を追加する。
-- HIGH-BIZ-002 監査対応: 003 マイグレーションで RLS は設定されているが、
-- FORCE ROW LEVEL SECURITY・AS RESTRICTIVE・WITH CHECK が欠落していた。
-- テーブルオーナーにも RLS を適用し、INSERT/UPDATE 時のテナント検証を強制する。

BEGIN;

SET LOCAL search_path TO activity_service, public;

-- activities テーブルに FORCE ROW LEVEL SECURITY を追加する
ALTER TABLE activity_service.activities FORCE ROW LEVEL SECURITY;

-- activities の tenant_isolation ポリシーを AS RESTRICTIVE + WITH CHECK 付きで再作成する
DROP POLICY IF EXISTS tenant_isolation ON activity_service.activities;
CREATE POLICY tenant_isolation ON activity_service.activities
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

COMMIT;
