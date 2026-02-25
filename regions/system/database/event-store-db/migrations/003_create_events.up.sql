-- event-store-db: events テーブル作成

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
