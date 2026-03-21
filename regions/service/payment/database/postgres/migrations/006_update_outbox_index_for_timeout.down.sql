-- 006_update_outbox_index_for_timeout.up.sql のロールバック
DROP INDEX IF EXISTS idx_outbox_events_unprocessed;
CREATE INDEX IF NOT EXISTS idx_outbox_events_unprocessed
    ON outbox_events (created_at ASC)
    WHERE published_at IS NULL AND processing_at IS NULL;
