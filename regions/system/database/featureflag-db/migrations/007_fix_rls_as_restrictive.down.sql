-- featureflag テーブルの AS RESTRICTIVE を元に戻す（006 マイグレーション相当に戻す）。

BEGIN;

SET LOCAL search_path TO featureflag, public;

-- feature_flags テーブルのポリシーを AS RESTRICTIVE なしで再作成する（006 相当）
DROP POLICY IF EXISTS tenant_isolation ON featureflag.feature_flags;
CREATE POLICY tenant_isolation ON featureflag.feature_flags
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- flag_audit_logs テーブルのポリシーを AS RESTRICTIVE なしで再作成する（006 相当）
DROP POLICY IF EXISTS tenant_isolation ON featureflag.flag_audit_logs;
CREATE POLICY tenant_isolation ON featureflag.flag_audit_logs
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

COMMIT;
