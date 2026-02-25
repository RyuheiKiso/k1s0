CREATE TABLE IF NOT EXISTS vault.secret_versions (
    id             UUID    PRIMARY KEY DEFAULT gen_random_uuid(),
    secret_id      UUID    NOT NULL REFERENCES vault.secrets(id) ON DELETE CASCADE,
    version        INT     NOT NULL,
    encrypted_data BYTEA   NOT NULL,
    nonce          BYTEA   NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_secret_versions_secret_version UNIQUE (secret_id, version)
);

CREATE INDEX IF NOT EXISTS idx_secret_versions_secret_id ON vault.secret_versions (secret_id);
