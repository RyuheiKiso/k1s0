ALTER TABLE quota.quota_usage
    ADD COLUMN IF NOT EXISTS subject_type VARCHAR(50) NOT NULL DEFAULT 'tenant';

ALTER TABLE quota.quota_usage
    ADD COLUMN IF NOT EXISTS subject_id VARCHAR(255) NOT NULL DEFAULT '';

UPDATE quota.quota_usage
SET subject_id = tenant_id
WHERE subject_id = '';

DROP INDEX IF EXISTS quota.idx_quota_usage_policy_tenant;

CREATE UNIQUE INDEX IF NOT EXISTS idx_quota_usage_policy_subject
    ON quota.quota_usage (policy_id, subject_type, subject_id);
