-- config-db: config_entries テーブル作成

CREATE TABLE IF NOT EXISTS config.config_entries (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    namespace   VARCHAR(255) NOT NULL,
    key         VARCHAR(255) NOT NULL,
    value_json  JSONB        NOT NULL DEFAULT '{}',
    version     INT          NOT NULL DEFAULT 1,
    description TEXT,
    created_by  VARCHAR(255) NOT NULL,
    updated_by  VARCHAR(255) NOT NULL,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_config_entries_namespace_key UNIQUE (namespace, key)
);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_config_entries_namespace
    ON config.config_entries (namespace);
CREATE INDEX IF NOT EXISTS idx_config_entries_namespace_key
    ON config.config_entries (namespace, key);
CREATE INDEX IF NOT EXISTS idx_config_entries_created_at
    ON config.config_entries (created_at);

-- updated_at トリガー
CREATE TRIGGER trigger_config_entries_update_updated_at
    BEFORE UPDATE ON config.config_entries
    FOR EACH ROW
    EXECUTE FUNCTION config.update_updated_at();
