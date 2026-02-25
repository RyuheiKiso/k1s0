CREATE TABLE IF NOT EXISTS quota.quota_policies (
    id            UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    name          VARCHAR(255) NOT NULL UNIQUE,
    subject_type  VARCHAR(50)  NOT NULL,
    subject_id    VARCHAR(255) NOT NULL DEFAULT '',
    quota_limit   BIGINT       NOT NULL,
    period        VARCHAR(50)  NOT NULL,
    enabled       BOOLEAN      NOT NULL DEFAULT true,
    alert_threshold_percent DOUBLE PRECISION NOT NULL DEFAULT 80.0,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_quota_policies_subject_type CHECK (subject_type IN ('tenant', 'user', 'api_key')),
    CONSTRAINT chk_quota_policies_period CHECK (period IN ('daily', 'monthly'))
);

CREATE INDEX IF NOT EXISTS idx_quota_policies_name ON quota.quota_policies (name);
CREATE INDEX IF NOT EXISTS idx_quota_policies_subject ON quota.quota_policies (subject_type, subject_id);

CREATE TRIGGER trigger_quota_policies_update_updated_at
    BEFORE UPDATE ON quota.quota_policies
    FOR EACH ROW
    EXECUTE FUNCTION quota.update_updated_at();
