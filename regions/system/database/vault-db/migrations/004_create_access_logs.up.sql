CREATE TABLE IF NOT EXISTS vault.access_logs (
    id         UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    secret_id  UUID,
    key_path   VARCHAR(512) NOT NULL,
    action     VARCHAR(50)  NOT NULL,
    actor_id   VARCHAR(255) NOT NULL DEFAULT '',
    ip_address VARCHAR(45),
    success    BOOLEAN      NOT NULL DEFAULT true,
    error_msg  TEXT,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_access_logs_action CHECK (action IN ('read', 'write', 'delete', 'list'))
);

CREATE INDEX IF NOT EXISTS idx_access_logs_key_path ON vault.access_logs (key_path);
CREATE INDEX IF NOT EXISTS idx_access_logs_created_at ON vault.access_logs (created_at DESC);
