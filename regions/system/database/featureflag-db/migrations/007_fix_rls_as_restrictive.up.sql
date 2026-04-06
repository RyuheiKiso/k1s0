-- featureflag テーブルの RLS ポリシーに AS RESTRICTIVE を追加する。
-- HIGH-DB-003 監査対応: 006 マイグレーションで WITH CHECK は追加されたが AS RESTRICTIVE が欠落していた。
-- AS RESTRICTIVE により他の PERMISSIVE ポリシーが存在しても必ずこのポリシーで制限される。
-- 既存ポリシーを DROP してから AS RESTRICTIVE 付きで再作成する。

BEGIN;

SET LOCAL search_path TO featureflag, public;

-- feature_flags テーブルの tenant_isolation ポリシーを AS RESTRICTIVE 付きで再作成する
DROP POLICY IF EXISTS tenant_isolation ON featureflag.feature_flags;
CREATE POLICY tenant_isolation ON featureflag.feature_flags
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- flag_audit_logs テーブルの tenant_isolation ポリシーを AS RESTRICTIVE 付きで再作成する
DROP POLICY IF EXISTS tenant_isolation ON featureflag.flag_audit_logs;
CREATE POLICY tenant_isolation ON featureflag.flag_audit_logs
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

COMMIT;
