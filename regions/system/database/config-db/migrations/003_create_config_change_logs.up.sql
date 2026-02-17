-- config-db: config_change_logs テーブル作成

CREATE TABLE IF NOT EXISTS config.config_change_logs (
    id               UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    config_entry_id  UUID         REFERENCES config.config_entries(id) ON DELETE SET NULL,
    namespace        VARCHAR(255) NOT NULL,
    key              VARCHAR(255) NOT NULL,
    change_type      VARCHAR(20)  NOT NULL,
    old_value_json   JSONB,
    new_value_json   JSONB,
    changed_by       VARCHAR(255) NOT NULL,
    trace_id         VARCHAR(64),
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_config_change_logs_change_type
        CHECK (change_type IN ('CREATED', 'UPDATED', 'DELETED'))
);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_config_change_logs_config_entry_id
    ON config.config_change_logs (config_entry_id);
CREATE INDEX IF NOT EXISTS idx_config_change_logs_namespace_key
    ON config.config_change_logs (namespace, key);
CREATE INDEX IF NOT EXISTS idx_config_change_logs_change_type_created_at
    ON config.config_change_logs (change_type, created_at);
CREATE INDEX IF NOT EXISTS idx_config_change_logs_trace_id
    ON config.config_change_logs (trace_id)
    WHERE trace_id IS NOT NULL;
