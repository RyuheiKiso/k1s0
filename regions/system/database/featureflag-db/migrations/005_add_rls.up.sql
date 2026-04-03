-- featureflag テーブルに RLS ポリシーを追加する
-- RUST-HIGH-002 対応: DB 層でのテナント分離を保証する
-- tenant_id は UUID 型（004_add_tenant_id.up.sql で定義）のため ::TEXT キャストで文字列比較する
SET search_path TO featureflag;

-- feature_flags テーブルの RLS 有効化
ALTER TABLE featureflag.feature_flags ENABLE ROW LEVEL SECURITY;
ALTER TABLE featureflag.feature_flags FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON featureflag.feature_flags;
CREATE POLICY tenant_isolation ON featureflag.feature_flags
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

-- flag_audit_logs テーブルの RLS 有効化
ALTER TABLE featureflag.flag_audit_logs ENABLE ROW LEVEL SECURITY;
ALTER TABLE featureflag.flag_audit_logs FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON featureflag.flag_audit_logs;
CREATE POLICY tenant_isolation ON featureflag.flag_audit_logs
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);
