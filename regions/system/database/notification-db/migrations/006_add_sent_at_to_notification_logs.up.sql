ALTER TABLE notification.notification_logs
    ADD COLUMN IF NOT EXISTS sent_at TIMESTAMPTZ;
