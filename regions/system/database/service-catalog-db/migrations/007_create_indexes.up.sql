-- Services indexes
CREATE INDEX IF NOT EXISTS idx_services_team_id ON service_catalog.services(team_id);
CREATE INDEX IF NOT EXISTS idx_services_tier ON service_catalog.services(tier);
CREATE INDEX IF NOT EXISTS idx_services_lifecycle ON service_catalog.services(lifecycle);
CREATE INDEX IF NOT EXISTS idx_services_name ON service_catalog.services(name);
CREATE INDEX IF NOT EXISTS idx_services_tags ON service_catalog.services USING GIN (tags);
CREATE INDEX IF NOT EXISTS idx_services_name_desc_search ON service_catalog.services USING GIN (
    to_tsvector('english', coalesce(name, '') || ' ' || coalesce(description, ''))
);

-- Dependencies indexes
CREATE INDEX IF NOT EXISTS idx_dependencies_source ON service_catalog.dependencies(source_service_id);
CREATE INDEX IF NOT EXISTS idx_dependencies_target ON service_catalog.dependencies(target_service_id);

-- Service docs indexes
CREATE INDEX IF NOT EXISTS idx_service_docs_service_id ON service_catalog.service_docs(service_id);

-- Health status indexes
CREATE INDEX IF NOT EXISTS idx_health_status_checked_at ON service_catalog.health_status(checked_at);
