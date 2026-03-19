-- L-8: ON DELETE SET NULL を元に戻す（デフォルトの ON DELETE NO ACTION に復元）

-- ON DELETE SET NULL 付きの外部キー制約を削除する
ALTER TABLE notification.notification_logs
    DROP CONSTRAINT IF EXISTS notification_logs_template_id_fkey;

-- デフォルト動作（ON DELETE NO ACTION）で外部キー制約を再作成する
ALTER TABLE notification.notification_logs
    ADD CONSTRAINT notification_logs_template_id_fkey
    FOREIGN KEY (template_id) REFERENCES notification.templates(id);
