-- HIGH-005対応: config-db RLS ポリシーのロールバック
-- RLS ポリシーを削除し、RLS を無効化する

SET LOCAL search_path TO config, public;

-- config_entries のテナント分離ポリシーを削除し RLS を無効化する
DROP POLICY IF EXISTS tenant_isolation ON config.config_entries;
ALTER TABLE config.config_entries DISABLE ROW LEVEL SECURITY;

-- config_change_logs のテナント分離ポリシーを削除し RLS を無効化する
DROP POLICY IF EXISTS tenant_isolation ON config.config_change_logs;
ALTER TABLE config.config_change_logs DISABLE ROW LEVEL SECURITY;

-- service_config_mappings の RLS を無効化する（tenant_id カラムが存在する場合）
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'config'
          AND table_name   = 'service_config_mappings'
          AND column_name  = 'tenant_id'
    ) THEN
        DROP POLICY IF EXISTS tenant_isolation ON config.service_config_mappings;
        ALTER TABLE config.service_config_mappings DISABLE ROW LEVEL SECURITY;
    END IF;
END $$;
