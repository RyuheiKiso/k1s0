DROP INDEX IF EXISTS quota.idx_quota_usage_policy_subject;

CREATE UNIQUE INDEX IF NOT EXISTS idx_quota_usage_policy_tenant
    ON quota.quota_usage (policy_id, tenant_id);

ALTER TABLE quota.quota_usage
    DROP COLUMN IF EXISTS subject_id;

ALTER TABLE quota.quota_usage
    DROP COLUMN IF EXISTS subject_type;
