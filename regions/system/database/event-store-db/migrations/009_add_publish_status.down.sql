-- publish_status カラムの削除
DROP INDEX IF EXISTS idx_events_publish_status_failed;
ALTER TABLE eventstore.events DROP COLUMN IF EXISTS publish_status;
