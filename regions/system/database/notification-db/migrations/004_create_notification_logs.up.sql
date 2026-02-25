CREATE TABLE IF NOT EXISTS notification.notification_logs (
    id            UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    channel_id    UUID         NOT NULL REFERENCES notification.channels(id),
    template_id   UUID         REFERENCES notification.templates(id),
    recipient     TEXT         NOT NULL,
    subject       TEXT         NOT NULL DEFAULT '',
    body          TEXT         NOT NULL DEFAULT '',
    status        VARCHAR(50)  NOT NULL DEFAULT 'pending',
    error_message TEXT,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_notification_logs_status CHECK (status IN ('pending', 'sent', 'failed'))
);

CREATE INDEX IF NOT EXISTS idx_notification_logs_channel_id ON notification.notification_logs (channel_id);
CREATE INDEX IF NOT EXISTS idx_notification_logs_status ON notification.notification_logs (status);
CREATE INDEX IF NOT EXISTS idx_notification_logs_created_at ON notification.notification_logs (created_at DESC);

CREATE TRIGGER trigger_notification_logs_update_updated_at
    BEFORE UPDATE ON notification.notification_logs
    FOR EACH ROW
    EXECUTE FUNCTION notification.update_updated_at();
