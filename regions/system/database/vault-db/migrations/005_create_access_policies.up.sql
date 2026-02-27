CREATE TABLE IF NOT EXISTS vault.access_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    secret_path_pattern VARCHAR(1024) NOT NULL,
    allowed_spiffe_ids TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_access_policies_path ON vault.access_policies (secret_path_pattern);
