CREATE TABLE IF NOT EXISTS tenant.tenant_members (
    id        UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID         NOT NULL REFERENCES tenant.tenants(id) ON DELETE CASCADE,
    user_id   UUID         NOT NULL,
    role      VARCHAR(50)  NOT NULL DEFAULT 'member',
    joined_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_members_role CHECK (role IN ('owner', 'admin', 'member', 'viewer')),
    CONSTRAINT uq_tenant_members_tenant_user UNIQUE (tenant_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_tenant_members_tenant_id ON tenant.tenant_members (tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_members_user_id ON tenant.tenant_members (user_id);
