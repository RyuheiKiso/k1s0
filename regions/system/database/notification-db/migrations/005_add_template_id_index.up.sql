-- notification_logs テーブルの template_id カラムにインデックスを追加する。
-- テンプレートID による通知ログの検索を高速化する。
CREATE INDEX IF NOT EXISTS idx_notification_logs_template_id
    ON notification.notification_logs (template_id);
