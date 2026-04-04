-- config-db の tenant_id を UUID から TEXT に変更する
-- C-004 監査対応: 全サービスで tenant_id を TEXT 型に統一する（ADR-0093）
-- UUID 値は文字列として透過的に保持される（'xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx' 形式）
-- current_setting('app.current_tenant_id') の TEXT 戻り値との型不一致を解消する
SET LOCAL search_path TO config, public;

-- config_entries テーブルの tenant_id を TEXT 型に変更する
-- USING 句で既存 UUID 値を文字列に変換し、デフォルト値を他サービスと統一する
ALTER TABLE config.config_entries
    ALTER COLUMN tenant_id TYPE TEXT USING tenant_id::TEXT;
ALTER TABLE config.config_entries
    ALTER COLUMN tenant_id SET DEFAULT 'system';

-- config_change_logs テーブルの tenant_id を TEXT 型に変更する
-- 変更ログも config_entries と同一の型・デフォルト値に統一する
ALTER TABLE config.config_change_logs
    ALTER COLUMN tenant_id TYPE TEXT USING tenant_id::TEXT;
ALTER TABLE config.config_change_logs
    ALTER COLUMN tenant_id SET DEFAULT 'system';
