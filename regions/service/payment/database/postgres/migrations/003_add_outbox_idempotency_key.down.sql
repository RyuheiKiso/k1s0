-- idempotency_key カラムを削除
DROP INDEX IF EXISTS idx_outbox_events_idempotency_key;
ALTER TABLE outbox_events DROP COLUMN IF EXISTS idempotency_key;
