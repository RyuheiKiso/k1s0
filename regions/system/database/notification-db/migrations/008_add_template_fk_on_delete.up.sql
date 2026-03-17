-- L-8: notification_logs の template_id 外部キーに ON DELETE SET NULL を追加する
-- テンプレート削除時に通知ログが孤立しないようにする

-- 既存の外部キー制約を削除する
ALTER TABLE notification.notification_logs
    DROP CONSTRAINT IF EXISTS notification_logs_template_id_fkey;

-- ON DELETE SET NULL 付きで外部キー制約を再作成する
ALTER TABLE notification.notification_logs
    ADD CONSTRAINT notification_logs_template_id_fkey
    FOREIGN KEY (template_id) REFERENCES notification.templates(id)
    ON DELETE SET NULL;
