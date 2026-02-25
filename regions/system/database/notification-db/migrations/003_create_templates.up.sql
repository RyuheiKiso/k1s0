CREATE TABLE IF NOT EXISTS notification.templates (
    id               UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    name             VARCHAR(255) NOT NULL,
    channel_type     VARCHAR(50)  NOT NULL,
    subject_template TEXT         NOT NULL DEFAULT '',
    body_template    TEXT         NOT NULL,
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_templates_channel_type ON notification.templates (channel_type);

CREATE TRIGGER trigger_templates_update_updated_at
    BEFORE UPDATE ON notification.templates
    FOR EACH ROW
    EXECUTE FUNCTION notification.update_updated_at();
