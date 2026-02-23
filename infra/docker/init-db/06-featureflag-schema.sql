-- infra/docker/init-db/06-featureflag-schema.sql

\c featureflag_db;

CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE SCHEMA IF NOT EXISTS featureflag;

CREATE OR REPLACE FUNCTION featureflag.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TABLE IF NOT EXISTS featureflag.feature_flags (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    flag_key    VARCHAR(255) UNIQUE NOT NULL,
    description TEXT,
    enabled     BOOLEAN      NOT NULL DEFAULT false,
    variants    JSONB        NOT NULL DEFAULT '[]',
    rules       JSONB        NOT NULL DEFAULT '[]',
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_feature_flags_flag_key ON featureflag.feature_flags (flag_key);
CREATE INDEX IF NOT EXISTS idx_feature_flags_enabled ON featureflag.feature_flags (enabled);

CREATE TRIGGER trigger_feature_flags_update_updated_at
    BEFORE UPDATE ON featureflag.feature_flags
    FOR EACH ROW EXECUTE FUNCTION featureflag.update_updated_at();

CREATE TABLE IF NOT EXISTS featureflag.flag_evaluations (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    flag_id     UUID         NOT NULL REFERENCES featureflag.feature_flags(id) ON DELETE CASCADE,
    user_id     VARCHAR(255),
    tenant_id   VARCHAR(255),
    result      BOOLEAN      NOT NULL,
    variant     VARCHAR(255),
    reason      VARCHAR(255),
    context     JSONB,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_flag_evaluations_flag_id ON featureflag.flag_evaluations (flag_id);
CREATE INDEX IF NOT EXISTS idx_flag_evaluations_user_id ON featureflag.flag_evaluations (user_id);
CREATE INDEX IF NOT EXISTS idx_flag_evaluations_created_at ON featureflag.flag_evaluations (created_at);

CREATE TABLE IF NOT EXISTS featureflag.flag_audit_logs (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    flag_id     UUID         REFERENCES featureflag.feature_flags(id) ON DELETE SET NULL,
    flag_key    VARCHAR(255) NOT NULL,
    action      VARCHAR(100) NOT NULL,
    changed_by  VARCHAR(255),
    old_value   JSONB,
    new_value   JSONB,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_flag_audit_logs_flag_id ON featureflag.flag_audit_logs (flag_id);
CREATE INDEX IF NOT EXISTS idx_flag_audit_logs_created_at ON featureflag.flag_audit_logs (created_at);
