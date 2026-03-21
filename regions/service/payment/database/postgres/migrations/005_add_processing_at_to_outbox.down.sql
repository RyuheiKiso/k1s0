DROP INDEX IF EXISTS idx_outbox_events_unprocessed;
ALTER TABLE outbox_events DROP COLUMN IF EXISTS processing_at;
