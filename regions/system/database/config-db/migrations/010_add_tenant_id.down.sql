-- STATIC-CRITICAL-001 監査対応: テナント分離カラムのロールバック

-- 新規インデックスを削除する
DROP INDEX IF EXISTS config.idx_config_entries_tenant_namespace;
DROP INDEX IF EXISTS config.idx_config_entries_tenant_id;
DROP INDEX IF EXISTS config.idx_config_change_logs_tenant_id;

-- 新規 UNIQUE 制約を削除して元の制約に戻す
ALTER TABLE config.config_entries
    DROP CONSTRAINT IF EXISTS uq_config_entries_tenant_namespace_key;

ALTER TABLE config.config_entries
    ADD CONSTRAINT uq_config_entries_namespace_key UNIQUE (namespace, key);

-- tenant_id カラムを削除する
ALTER TABLE config.config_entries DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE config.config_change_logs DROP COLUMN IF EXISTS tenant_id;
