-- featureflag の tenant_id を UUID から TEXT に変更する
-- C-004 監査対応: 全サービスで tenant_id を TEXT 型に統一する（ADR-0093）
-- UUID 値は文字列として透過的に保持される（'xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx' 形式）
-- RLS ポリシーの ::TEXT キャストが不要になるが後方互換性のため残す
SET LOCAL search_path TO featureflag, public;

-- feature_flags テーブルの tenant_id を TEXT 型に変更する
-- USING 句で既存 UUID 値を文字列に変換し、デフォルト値を他サービスと統一する
ALTER TABLE featureflag.feature_flags
    ALTER COLUMN tenant_id TYPE TEXT USING tenant_id::TEXT;
ALTER TABLE featureflag.feature_flags
    ALTER COLUMN tenant_id SET DEFAULT 'system';

-- flag_audit_logs テーブルの tenant_id を TEXT 型に変更する
-- 監査ログも feature_flags と同一の型・デフォルト値に統一する
ALTER TABLE featureflag.flag_audit_logs
    ALTER COLUMN tenant_id TYPE TEXT USING tenant_id::TEXT;
ALTER TABLE featureflag.flag_audit_logs
    ALTER COLUMN tenant_id SET DEFAULT 'system';
