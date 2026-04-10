-- config.config_schemas と config.service_config_mappings にテナント分離を実装する。
-- HIGH-DB-001 監査対応: config_entries / config_change_logs は 012/013 で対応済みだが、
-- config_schemas と service_config_mappings は未対応のためテナント分離を追加する。
-- service_config_mappings は tenant_id カラムが存在しない場合にのみ追加する。
-- 既存データは 'system' テナントとしてバックフィルし、新規挿入を強制する。

BEGIN;

SET LOCAL search_path TO config, public;

-- config_schemas テーブルに tenant_id カラムを追加する
ALTER TABLE config.config_schemas
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

-- バックフィル後はデフォルト値を削除し、新規挿入時の明示指定を強制する
ALTER TABLE config.config_schemas
    ALTER COLUMN tenant_id DROP DEFAULT;

-- tenant_id のインデックスを追加する
CREATE INDEX IF NOT EXISTS idx_config_schemas_tenant_id
    ON config.config_schemas (tenant_id);

-- 既存の UNIQUE(service_name) 制約を削除し、テナントスコープの UNIQUE に変更する
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'config_schemas_service_name_key' AND conrelid = 'config.config_schemas'::regclass
    ) THEN
        ALTER TABLE config.config_schemas DROP CONSTRAINT config_schemas_service_name_key;
    END IF;
END $$;
CREATE UNIQUE INDEX IF NOT EXISTS uq_config_schemas_tenant_service_name
    ON config.config_schemas (tenant_id, service_name);

-- config_schemas テーブルの RLS を有効化する
ALTER TABLE config.config_schemas ENABLE ROW LEVEL SECURITY;
ALTER TABLE config.config_schemas FORCE ROW LEVEL SECURITY;

-- テナント分離ポリシーを設定する
DROP POLICY IF EXISTS tenant_isolation ON config.config_schemas;
CREATE POLICY tenant_isolation ON config.config_schemas
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- service_config_mappings テーブルに tenant_id カラムが存在しない場合のみ追加する
-- （013 マイグレーションで条件付き追加が実施されている可能性があるため二重追加を防ぐ）
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'config'
          AND table_name = 'service_config_mappings'
          AND column_name = 'tenant_id'
    ) THEN
        ALTER TABLE config.service_config_mappings
            ADD COLUMN tenant_id TEXT NOT NULL DEFAULT 'system';
        ALTER TABLE config.service_config_mappings
            ALTER COLUMN tenant_id DROP DEFAULT;
        CREATE INDEX IF NOT EXISTS idx_service_config_mappings_tenant_id
            ON config.service_config_mappings (tenant_id);
    END IF;
END $$;

-- service_config_mappings テーブルの RLS を有効化する
ALTER TABLE config.service_config_mappings ENABLE ROW LEVEL SECURITY;
ALTER TABLE config.service_config_mappings FORCE ROW LEVEL SECURITY;

-- テナント分離ポリシーを設定する（既存ポリシーがあれば削除してから再作成）
DROP POLICY IF EXISTS tenant_isolation ON config.service_config_mappings;
CREATE POLICY tenant_isolation ON config.service_config_mappings
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

COMMIT;
