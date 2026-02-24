CREATE TABLE apiregistry.api_schema_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL REFERENCES apiregistry.api_schemas(name) ON DELETE CASCADE,
    version INT NOT NULL,
    schema_type VARCHAR(50) NOT NULL,
    content TEXT NOT NULL,
    content_hash VARCHAR(255) NOT NULL,
    breaking_changes BOOLEAN NOT NULL DEFAULT FALSE,
    registered_by VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_schema_version UNIQUE (name, version)
);
