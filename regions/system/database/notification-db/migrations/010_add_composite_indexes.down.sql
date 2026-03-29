BEGIN;
DROP INDEX IF EXISTS notification.idx_notification_logs_channel_status;
DROP INDEX IF EXISTS notification.idx_notification_logs_status_created_at;
COMMIT;
