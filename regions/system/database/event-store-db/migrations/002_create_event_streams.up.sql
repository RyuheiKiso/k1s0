-- event-store-db: event_streams テーブル作成

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
