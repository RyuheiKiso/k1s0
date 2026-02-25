ALTER TABLE apiregistry.api_schema_versions
ADD COLUMN breaking_change_details JSONB NOT NULL DEFAULT '[]';
