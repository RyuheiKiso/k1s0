-- infra/docker/init-db/09-vault-schema.sql

\c vault_db;

CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE SCHEMA IF NOT EXISTS vault;

CREATE TABLE IF NOT EXISTS vault.secret_access_logs (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    path        VARCHAR(1024) NOT NULL,
    action      VARCHAR(50)  NOT NULL,
    subject     VARCHAR(255),
    tenant_id   VARCHAR(255),
    ip_address  INET,
    user_agent  TEXT,
    trace_id    VARCHAR(64),
    success     BOOLEAN      NOT NULL DEFAULT true,
    error_msg   TEXT,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_vault_access_action CHECK (action IN ('read', 'write', 'delete', 'list'))
);

CREATE INDEX IF NOT EXISTS idx_secret_access_logs_path ON vault.secret_access_logs (path);
CREATE INDEX IF NOT EXISTS idx_secret_access_logs_subject ON vault.secret_access_logs (subject);
CREATE INDEX IF NOT EXISTS idx_secret_access_logs_created_at ON vault.secret_access_logs (created_at);
CREATE INDEX IF NOT EXISTS idx_secret_access_logs_trace_id ON vault.secret_access_logs (trace_id)
    WHERE trace_id IS NOT NULL;
