-- featureflag RLS の逆マイグレーション
-- RUST-HIGH-002 対応の取り消し: RLS ポリシーを削除してテーブルへの全行アクセスを復元する
SET search_path TO featureflag;

DROP POLICY IF EXISTS tenant_isolation ON featureflag.flag_audit_logs;
ALTER TABLE featureflag.flag_audit_logs DISABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON featureflag.feature_flags;
ALTER TABLE featureflag.feature_flags DISABLE ROW LEVEL SECURITY;
