-- infra/docker/init-db/03-config-schema.sql
-- config-db マイグレーション結合（regions/system/database/config-db/migrations/ の全 .up.sql）

\c config_db;

-- 001: スキーマ・拡張機能・共通関数
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE SCHEMA IF NOT EXISTS config;

CREATE OR REPLACE FUNCTION config.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 002: config_entries テーブル
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

CREATE INDEX IF NOT EXISTS idx_config_entries_namespace
    ON config.config_entries (namespace);
CREATE INDEX IF NOT EXISTS idx_config_entries_namespace_key
    ON config.config_entries (namespace, key);
CREATE INDEX IF NOT EXISTS idx_config_entries_created_at
    ON config.config_entries (created_at);

CREATE TRIGGER trigger_config_entries_update_updated_at
    BEFORE UPDATE ON config.config_entries
    FOR EACH ROW
    EXECUTE FUNCTION config.update_updated_at();

-- 003: config_change_logs テーブル
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

CREATE INDEX IF NOT EXISTS idx_config_change_logs_config_entry_id
    ON config.config_change_logs (config_entry_id);
CREATE INDEX IF NOT EXISTS idx_config_change_logs_namespace_key
    ON config.config_change_logs (namespace, key);
CREATE INDEX IF NOT EXISTS idx_config_change_logs_change_type_created_at
    ON config.config_change_logs (change_type, created_at);
CREATE INDEX IF NOT EXISTS idx_config_change_logs_trace_id
    ON config.config_change_logs (trace_id)
    WHERE trace_id IS NOT NULL;

-- 004: 初期データ投入
INSERT INTO config.config_entries (namespace, key, value_json, description, created_by, updated_by) VALUES
    ('system.auth.database', 'host',     '"localhost"',           'DB ホスト名',          'migration', 'migration'),
    ('system.auth.database', 'port',     '5432',                 'DB ポート番号',        'migration', 'migration'),
    ('system.auth.database', 'name',     '"k1s0_system"',        'DB 名',               'migration', 'migration'),
    ('system.auth.database', 'ssl_mode', '"disable"',            'SSL モード',           'migration', 'migration')
ON CONFLICT (namespace, key) DO NOTHING;

INSERT INTO config.config_entries (namespace, key, value_json, description, created_by, updated_by) VALUES
    ('system.auth.server', 'port',          '8081',   'gRPC リッスンポート',     'migration', 'migration'),
    ('system.auth.server', 'read_timeout',  '30',     '読み取りタイムアウト（秒）', 'migration', 'migration'),
    ('system.auth.server', 'write_timeout', '30',     '書き込みタイムアウト（秒）', 'migration', 'migration')
ON CONFLICT (namespace, key) DO NOTHING;

INSERT INTO config.config_entries (namespace, key, value_json, description, created_by, updated_by) VALUES
    ('system.config.database', 'host',     '"localhost"',           'DB ホスト名',          'migration', 'migration'),
    ('system.config.database', 'port',     '5432',                 'DB ポート番号',        'migration', 'migration'),
    ('system.config.database', 'name',     '"k1s0_system"',        'DB 名',               'migration', 'migration'),
    ('system.config.database', 'ssl_mode', '"disable"',            'SSL モード',           'migration', 'migration')
ON CONFLICT (namespace, key) DO NOTHING;

INSERT INTO config.config_entries (namespace, key, value_json, description, created_by, updated_by) VALUES
    ('system.config.server', 'port',          '8082',   'gRPC リッスンポート',     'migration', 'migration'),
    ('system.config.server', 'read_timeout',  '30',     '読み取りタイムアウト（秒）', 'migration', 'migration'),
    ('system.config.server', 'write_timeout', '30',     '書き込みタイムアウト（秒）', 'migration', 'migration')
ON CONFLICT (namespace, key) DO NOTHING;
