-- infra/docker/init-db/12-event-store-schema.sql
-- event-store-db マイグレーション結合（regions/system/database/event-store-db/migrations/ の全 .up.sql）

\c event_store_db;

-- 001: スキーマ・拡張機能・共通関数
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE SCHEMA IF NOT EXISTS eventstore;

CREATE OR REPLACE FUNCTION eventstore.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 002: event_streams テーブル
CREATE TABLE IF NOT EXISTS eventstore.event_streams (
    id             UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_type VARCHAR(255) NOT NULL,
    current_version BIGINT      NOT NULL DEFAULT 0,
    metadata       JSONB        NOT NULL DEFAULT '{}',
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_event_streams_aggregate_type ON eventstore.event_streams (aggregate_type);

CREATE TRIGGER trigger_event_streams_update_updated_at
    BEFORE UPDATE ON eventstore.event_streams
    FOR EACH ROW
    EXECUTE FUNCTION eventstore.update_updated_at();

-- 003: events テーブル（Append-only: UPDATE/DELETE 禁止）
CREATE TABLE IF NOT EXISTS eventstore.events (
    id         UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    stream_id  UUID         NOT NULL REFERENCES eventstore.event_streams(id) ON DELETE CASCADE,
    sequence   BIGINT       NOT NULL,
    event_type VARCHAR(255) NOT NULL,
    version    BIGINT       NOT NULL DEFAULT 1,
    payload    JSONB        NOT NULL DEFAULT '{}',
    metadata   JSONB        NOT NULL DEFAULT '{}',
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    stored_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_events_stream_sequence UNIQUE (stream_id, sequence)
);

CREATE INDEX IF NOT EXISTS idx_events_stream_id ON eventstore.events (stream_id);
CREATE INDEX IF NOT EXISTS idx_events_event_type ON eventstore.events (event_type);
CREATE INDEX IF NOT EXISTS idx_events_stream_sequence ON eventstore.events (stream_id, sequence);

-- 004: snapshots テーブル
CREATE TABLE IF NOT EXISTS eventstore.snapshots (
    id               UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    stream_id        UUID         NOT NULL REFERENCES eventstore.event_streams(id) ON DELETE CASCADE,
    snapshot_version BIGINT       NOT NULL,
    aggregate_type   VARCHAR(255) NOT NULL DEFAULT '',
    state            JSONB        NOT NULL DEFAULT '{}',
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_snapshots_stream_id ON eventstore.snapshots (stream_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_snapshots_stream_version ON eventstore.snapshots (stream_id, snapshot_version);
