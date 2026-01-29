-- Outbox イベントテーブル
CREATE TABLE IF NOT EXISTS outbox_events (
    id UUID PRIMARY KEY,
    event_type VARCHAR(255) NOT NULL,
    payload JSONB NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at TIMESTAMPTZ,
    retry_count INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    CONSTRAINT chk_status CHECK (status IN ('pending', 'published', 'failed'))
);

-- 未処理エントリの高速取得用インデックス
CREATE INDEX IF NOT EXISTS idx_outbox_events_pending
    ON outbox_events (created_at)
    WHERE status = 'pending';

-- イベント型による検索用インデックス
CREATE INDEX IF NOT EXISTS idx_outbox_events_event_type
    ON outbox_events (event_type);
