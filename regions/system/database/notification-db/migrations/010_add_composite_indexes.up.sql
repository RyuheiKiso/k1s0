-- M-013 監査対応: notification-db に頻繁に使用されるクエリパターン向けの複合インデックスを追加する
-- (channel_id, status) および (status, created_at) の組み合わせクエリを最適化する

BEGIN;

-- notification_logs の channel_id + status による絞り込みクエリを最適化する
CREATE INDEX IF NOT EXISTS idx_notification_logs_channel_status
    ON notification.notification_logs (channel_id, status);

-- notification_logs の status + created_at による時系列クエリを最適化する
CREATE INDEX IF NOT EXISTS idx_notification_logs_status_created_at
    ON notification.notification_logs (status, created_at DESC);

COMMIT;
