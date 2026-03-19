CREATE TABLE IF NOT EXISTS service_catalog.dependencies (
    source_service_id UUID NOT NULL REFERENCES service_catalog.services(id) ON DELETE CASCADE,
    target_service_id UUID NOT NULL REFERENCES service_catalog.services(id) ON DELETE CASCADE,
    dependency_type VARCHAR(50) NOT NULL DEFAULT 'runtime',
    description TEXT,
    PRIMARY KEY (source_service_id, target_service_id)
);
