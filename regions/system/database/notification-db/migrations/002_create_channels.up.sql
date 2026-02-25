CREATE TABLE IF NOT EXISTS notification.channels (
    id           UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    name         VARCHAR(255) NOT NULL,
    channel_type VARCHAR(50)  NOT NULL,
    config       JSONB        NOT NULL DEFAULT '{}',
    enabled      BOOLEAN      NOT NULL DEFAULT true,
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_channels_channel_type ON notification.channels (channel_type);
CREATE INDEX IF NOT EXISTS idx_channels_enabled ON notification.channels (enabled);

CREATE TRIGGER trigger_channels_update_updated_at
    BEFORE UPDATE ON notification.channels
    FOR EACH ROW
    EXECUTE FUNCTION notification.update_updated_at();
