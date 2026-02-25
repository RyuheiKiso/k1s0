CREATE TABLE IF NOT EXISTS policy.policies (
    id           UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    name         VARCHAR(255) NOT NULL UNIQUE,
    description  TEXT         NOT NULL DEFAULT '',
    rego_content TEXT         NOT NULL,
    package_path VARCHAR(255) NOT NULL DEFAULT '',
    enabled      BOOLEAN      NOT NULL DEFAULT true,
    version      INT          NOT NULL DEFAULT 1,
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_policies_name ON policy.policies (name);
CREATE INDEX IF NOT EXISTS idx_policies_enabled ON policy.policies (enabled);

CREATE TRIGGER trigger_policies_update_updated_at
    BEFORE UPDATE ON policy.policies
    FOR EACH ROW
    EXECUTE FUNCTION policy.update_updated_at();
