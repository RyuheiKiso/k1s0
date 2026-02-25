CREATE TABLE IF NOT EXISTS vault.secrets (
    id              UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    key_path        VARCHAR(512) NOT NULL UNIQUE,
    current_version INT          NOT NULL DEFAULT 1,
    metadata        JSONB        NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_secrets_key_path ON vault.secrets (key_path);

CREATE TRIGGER trigger_secrets_update_updated_at
    BEFORE UPDATE ON vault.secrets
    FOR EACH ROW
    EXECUTE FUNCTION vault.update_updated_at();
