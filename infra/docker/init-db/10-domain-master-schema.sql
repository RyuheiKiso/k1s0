-- Domain Master (accounting business tier)
\c k1s0_business;

CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE SCHEMA IF NOT EXISTS domain_master;

CREATE TABLE domain_master.master_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(100) NOT NULL UNIQUE,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    validation_schema JSONB,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_by VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_master_categories_code ON domain_master.master_categories(code);
CREATE INDEX idx_master_categories_active ON domain_master.master_categories(is_active);

CREATE TABLE domain_master.master_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    category_id UUID NOT NULL REFERENCES domain_master.master_categories(id) ON DELETE CASCADE,
    code VARCHAR(255) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    attributes JSONB,
    parent_item_id UUID REFERENCES domain_master.master_items(id) ON DELETE SET NULL,
    effective_from TIMESTAMPTZ,
    effective_until TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_by VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_master_items_category_code UNIQUE (category_id, code)
);

CREATE INDEX idx_master_items_category ON domain_master.master_items(category_id);
CREATE INDEX idx_master_items_active ON domain_master.master_items(is_active);
CREATE INDEX idx_master_items_parent ON domain_master.master_items(parent_item_id);
CREATE INDEX idx_master_items_effective ON domain_master.master_items(effective_from, effective_until);

CREATE TABLE domain_master.master_item_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id UUID NOT NULL REFERENCES domain_master.master_items(id) ON DELETE CASCADE,
    version_number INTEGER NOT NULL,
    before_data JSONB,
    after_data JSONB,
    changed_by VARCHAR(255) NOT NULL,
    change_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_master_item_versions_item_version UNIQUE (item_id, version_number)
);

CREATE INDEX idx_master_item_versions_item ON domain_master.master_item_versions(item_id, created_at DESC);

CREATE TABLE domain_master.tenant_master_extensions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id VARCHAR(255) NOT NULL,
    item_id UUID NOT NULL REFERENCES domain_master.master_items(id) ON DELETE CASCADE,
    display_name_override VARCHAR(255),
    attributes_override JSONB,
    is_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_tenant_master_extensions_tenant_item UNIQUE (tenant_id, item_id)
);

CREATE INDEX idx_tenant_master_extensions_tenant ON domain_master.tenant_master_extensions(tenant_id);
CREATE INDEX idx_tenant_master_extensions_item ON domain_master.tenant_master_extensions(item_id);
