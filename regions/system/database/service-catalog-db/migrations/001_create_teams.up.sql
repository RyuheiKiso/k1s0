CREATE SCHEMA IF NOT EXISTS service_catalog;

CREATE TABLE IF NOT EXISTS service_catalog.teams (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    contact_email VARCHAR(255),
    slack_channel VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
