-- event-store-db: snapshots テーブル作成

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
