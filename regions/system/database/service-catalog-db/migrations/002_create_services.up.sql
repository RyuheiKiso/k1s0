CREATE TABLE service_catalog.services (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    team_id UUID NOT NULL REFERENCES service_catalog.teams(id) ON DELETE CASCADE,
    tier VARCHAR(50) NOT NULL DEFAULT 'standard',
    lifecycle VARCHAR(50) NOT NULL DEFAULT 'development',
    repository_url TEXT,
    api_endpoint TEXT,
    healthcheck_url TEXT,
    tags JSONB NOT NULL DEFAULT '[]'::jsonb,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
