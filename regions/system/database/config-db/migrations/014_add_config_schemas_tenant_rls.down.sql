-- config.config_schemas と config.service_config_mappings のテナント分離を元に戻す。

BEGIN;

SET LOCAL search_path TO config, public;

-- service_config_mappings の RLS 無効化とポリシー削除
DROP POLICY IF EXISTS tenant_isolation ON config.service_config_mappings;
ALTER TABLE config.service_config_mappings DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS config.idx_service_config_mappings_tenant_id;
ALTER TABLE config.service_config_mappings DROP COLUMN IF EXISTS tenant_id;

-- config_schemas の RLS 無効化とポリシー削除
DROP POLICY IF EXISTS tenant_isolation ON config.config_schemas;
ALTER TABLE config.config_schemas DISABLE ROW LEVEL SECURITY;

-- テナントスコープ UNIQUE INDEX を削除し、元の service_name UNIQUE 制約を復元する
DROP INDEX IF EXISTS config.uq_config_schemas_tenant_service_name;
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'config_schemas_service_name_key' AND conrelid = 'config.config_schemas'::regclass
    ) THEN
        ALTER TABLE config.config_schemas ADD CONSTRAINT config_schemas_service_name_key UNIQUE (service_name);
    END IF;
END $$;

DROP INDEX IF EXISTS config.idx_config_schemas_tenant_id;
ALTER TABLE config.config_schemas DROP COLUMN IF EXISTS tenant_id;

COMMIT;
