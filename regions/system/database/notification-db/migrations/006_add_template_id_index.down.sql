-- notification_logs テーブルの template_id インデックスを削除する
DROP INDEX IF EXISTS notification.idx_notification_logs_template_id;
