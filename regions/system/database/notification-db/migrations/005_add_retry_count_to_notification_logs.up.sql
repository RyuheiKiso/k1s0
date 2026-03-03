ALTER TABLE notification.notification_logs
    ADD COLUMN IF NOT EXISTS retry_count INT NOT NULL DEFAULT 0;

