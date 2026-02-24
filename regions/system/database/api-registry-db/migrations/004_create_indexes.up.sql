CREATE INDEX idx_api_schemas_schema_type ON apiregistry.api_schemas(schema_type);
CREATE INDEX idx_api_schemas_created_at ON apiregistry.api_schemas(created_at DESC);
CREATE INDEX idx_api_schema_versions_name ON apiregistry.api_schema_versions(name);
CREATE INDEX idx_api_schema_versions_name_version ON apiregistry.api_schema_versions(name, version DESC);
CREATE INDEX idx_api_schema_versions_created_at ON apiregistry.api_schema_versions(created_at DESC);
