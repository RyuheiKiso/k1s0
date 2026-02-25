CREATE TABLE IF NOT EXISTS tenant.tenants (
    id           UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    name         VARCHAR(255) NOT NULL UNIQUE,
    display_name VARCHAR(255) NOT NULL,
    status       VARCHAR(50)  NOT NULL DEFAULT 'provisioning',
    plan         VARCHAR(50)  NOT NULL DEFAULT 'free',
    settings     JSONB        NOT NULL DEFAULT '{}',
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_tenants_status CHECK (status IN ('provisioning', 'active', 'suspended', 'deleted')),
    CONSTRAINT chk_tenants_plan CHECK (plan IN ('free', 'starter', 'professional', 'enterprise'))
);

CREATE INDEX IF NOT EXISTS idx_tenants_name ON tenant.tenants (name);
CREATE INDEX IF NOT EXISTS idx_tenants_status ON tenant.tenants (status);

CREATE TRIGGER trigger_tenants_update_updated_at
    BEFORE UPDATE ON tenant.tenants
    FOR EACH ROW
    EXECUTE FUNCTION tenant.update_updated_at();
