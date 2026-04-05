-- STATIC-CRITICAL-001 監査対応: config_entries にテナント分離カラムを追加する
-- 既存データには システムテナント UUID (00000000-0000-0000-0000-000000000001) を割り当てる

ALTER TABLE config.config_entries
    ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001';

ALTER TABLE config.config_change_logs
    ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001';

-- 既存の UNIQUE 制約を削除して tenant_id を含む新しい制約に変更する
ALTER TABLE config.config_entries
    DROP CONSTRAINT IF EXISTS uq_config_entries_namespace_key;

ALTER TABLE config.config_entries
    ADD CONSTRAINT uq_config_entries_tenant_namespace_key UNIQUE (tenant_id, namespace, key);

-- テナントIDによる高速検索のためのインデックスを追加する
CREATE INDEX IF NOT EXISTS idx_config_entries_tenant_id
    ON config.config_entries (tenant_id);

CREATE INDEX IF NOT EXISTS idx_config_entries_tenant_namespace
    ON config.config_entries (tenant_id, namespace);

CREATE INDEX IF NOT EXISTS idx_config_change_logs_tenant_id
    ON config.config_change_logs (tenant_id);

-- ADD COLUMN 後は DEFAULT 制約を削除し、今後のINSERTで明示的に指定させる
ALTER TABLE config.config_entries ALTER COLUMN tenant_id DROP DEFAULT;
ALTER TABLE config.config_change_logs ALTER COLUMN tenant_id DROP DEFAULT;
