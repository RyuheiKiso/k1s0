CREATE TABLE IF NOT EXISTS quota.quota_usage (
    id                  UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    policy_id           UUID         NOT NULL REFERENCES quota.quota_policies(id) ON DELETE CASCADE,
    tenant_id           VARCHAR(255) NOT NULL DEFAULT '',
    current_usage       BIGINT       NOT NULL DEFAULT 0,
    window_start        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    last_incremented_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_quota_usage_policy_id ON quota.quota_usage (policy_id);
CREATE INDEX IF NOT EXISTS idx_quota_usage_window ON quota.quota_usage (window_start);
CREATE UNIQUE INDEX IF NOT EXISTS idx_quota_usage_policy_tenant ON quota.quota_usage (policy_id, tenant_id);
