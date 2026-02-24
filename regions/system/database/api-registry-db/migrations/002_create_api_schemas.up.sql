CREATE TABLE apiregistry.api_schemas (
    name VARCHAR(255) PRIMARY KEY,
    description TEXT NOT NULL DEFAULT '',
    schema_type VARCHAR(50) NOT NULL,
    latest_version INT NOT NULL DEFAULT 0,
    version_count INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT ck_schema_type CHECK (schema_type IN ('openapi', 'protobuf'))
);
