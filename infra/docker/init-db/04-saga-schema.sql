-- infra/docker/init-db/04-saga-schema.sql

\connect k1s0_system;

CREATE SCHEMA IF NOT EXISTS saga;

CREATE TABLE saga.saga_states (
    id UUID PRIMARY KEY,
    workflow_name VARCHAR(255) NOT NULL,
    current_step INT NOT NULL DEFAULT 0,
    status VARCHAR(50) NOT NULL DEFAULT 'STARTED',
    payload JSONB,
    correlation_id VARCHAR(255),
    initiated_by VARCHAR(255),
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE saga.saga_step_logs (
    id UUID PRIMARY KEY,
    saga_id UUID NOT NULL REFERENCES saga.saga_states(id),
    step_index INT NOT NULL,
    step_name VARCHAR(255) NOT NULL,
    action VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL,
    request_payload JSONB,
    response_payload JSONB,
    error_message TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_saga_states_status ON saga.saga_states(status);
CREATE INDEX idx_saga_states_workflow ON saga.saga_states(workflow_name);
CREATE INDEX idx_saga_states_correlation ON saga.saga_states(correlation_id);
CREATE INDEX idx_saga_step_logs_saga_id ON saga.saga_step_logs(saga_id);
