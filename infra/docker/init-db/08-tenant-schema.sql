-- infra/docker/init-db/08-tenant-schema.sql

\c tenant_db;

CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE SCHEMA IF NOT EXISTS tenant;

CREATE OR REPLACE FUNCTION tenant.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TABLE IF NOT EXISTS tenant.tenants (
    id           UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    name         VARCHAR(255) UNIQUE NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    status       VARCHAR(50)  NOT NULL DEFAULT 'provisioning',
    plan         VARCHAR(100) NOT NULL DEFAULT 'free',
    realm_name   VARCHAR(255),
    owner_id     UUID,
    metadata     JSONB        NOT NULL DEFAULT '{}',
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_tenants_status CHECK (status IN ('provisioning', 'active', 'suspended', 'deleted'))
);

CREATE INDEX IF NOT EXISTS idx_tenants_status ON tenant.tenants (status);
CREATE INDEX IF NOT EXISTS idx_tenants_name ON tenant.tenants (name);

CREATE TRIGGER trigger_tenants_update_updated_at
    BEFORE UPDATE ON tenant.tenants
    FOR EACH ROW EXECUTE FUNCTION tenant.update_updated_at();

CREATE TABLE IF NOT EXISTS tenant.tenant_members (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id   UUID         NOT NULL REFERENCES tenant.tenants(id) ON DELETE CASCADE,
    user_id     UUID         NOT NULL,
    role        VARCHAR(100) NOT NULL DEFAULT 'member',
    joined_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_tenant_members_tenant_user UNIQUE (tenant_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_tenant_members_tenant_id ON tenant.tenant_members (tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_members_user_id ON tenant.tenant_members (user_id);

CREATE TABLE IF NOT EXISTS tenant.tenant_provisioning_jobs (
    id            UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id     UUID         NOT NULL REFERENCES tenant.tenants(id) ON DELETE CASCADE,
    status        VARCHAR(50)  NOT NULL DEFAULT 'pending',
    current_step  VARCHAR(255),
    error_message TEXT,
    metadata      JSONB        NOT NULL DEFAULT '{}',
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_provisioning_status CHECK (status IN ('pending', 'running', 'completed', 'failed'))
);

CREATE INDEX IF NOT EXISTS idx_provisioning_jobs_tenant_id ON tenant.tenant_provisioning_jobs (tenant_id);
CREATE INDEX IF NOT EXISTS idx_provisioning_jobs_status ON tenant.tenant_provisioning_jobs (status);

CREATE TRIGGER trigger_provisioning_jobs_update_updated_at
    BEFORE UPDATE ON tenant.tenant_provisioning_jobs
    FOR EACH ROW EXECUTE FUNCTION tenant.update_updated_at();
