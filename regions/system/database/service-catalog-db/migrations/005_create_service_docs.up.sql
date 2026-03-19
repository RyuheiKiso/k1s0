CREATE TABLE IF NOT EXISTS service_catalog.service_docs (
    id UUID PRIMARY KEY,
    service_id UUID NOT NULL REFERENCES service_catalog.services(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    url TEXT NOT NULL,
    doc_type VARCHAR(50) NOT NULL DEFAULT 'other',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
