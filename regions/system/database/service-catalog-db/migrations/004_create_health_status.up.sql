CREATE TABLE service_catalog.health_status (
    service_id UUID PRIMARY KEY REFERENCES service_catalog.services(id) ON DELETE CASCADE,
    status VARCHAR(50) NOT NULL DEFAULT 'unknown',
    message TEXT,
    response_time_ms BIGINT,
    checked_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
