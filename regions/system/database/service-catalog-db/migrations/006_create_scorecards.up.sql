CREATE TABLE IF NOT EXISTS service_catalog.scorecards (
    service_id UUID PRIMARY KEY REFERENCES service_catalog.services(id) ON DELETE CASCADE,
    documentation_score DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    test_coverage_score DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    slo_compliance_score DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    security_score DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    overall_score DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    evaluated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
