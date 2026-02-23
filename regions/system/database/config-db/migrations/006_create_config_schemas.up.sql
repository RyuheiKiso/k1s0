-- config-db: config_schemas テーブル作成

CREATE TABLE IF NOT EXISTS config.config_schemas (
    id               UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    service_name     VARCHAR(255) NOT NULL UNIQUE,
    namespace_prefix VARCHAR(255) NOT NULL,
    schema_json      JSONB        NOT NULL,
    updated_by       VARCHAR(255) NOT NULL DEFAULT 'system',
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_config_schemas_service_name
    ON config.config_schemas (service_name);

-- updated_at 自動更新トリガー
CREATE TRIGGER trigger_config_schemas_update_updated_at
    BEFORE UPDATE ON config.config_schemas
    FOR EACH ROW EXECUTE FUNCTION config.update_updated_at();
