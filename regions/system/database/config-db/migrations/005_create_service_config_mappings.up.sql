-- config-db: service_config_mappings テーブル作成

CREATE TABLE IF NOT EXISTS config.service_config_mappings (
    id               UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    service_name     VARCHAR(255) NOT NULL,
    config_entry_id  UUID         NOT NULL REFERENCES config.config_entries(id) ON DELETE CASCADE,
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_service_config_mappings_service_entry UNIQUE (service_name, config_entry_id)
);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_service_config_mappings_service_name
    ON config.service_config_mappings (service_name);
CREATE INDEX IF NOT EXISTS idx_service_config_mappings_config_entry_id
    ON config.service_config_mappings (config_entry_id);
