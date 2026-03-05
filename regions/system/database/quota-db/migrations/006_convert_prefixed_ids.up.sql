ALTER TABLE quota.quota_usage
    DROP CONSTRAINT IF EXISTS quota_usage_policy_id_fkey;

ALTER TABLE quota.quota_policies
    ALTER COLUMN id DROP DEFAULT,
    ALTER COLUMN id TYPE VARCHAR(64)
    USING ('quota_' || replace(id::text, '-', ''));

ALTER TABLE quota.quota_usage
    ALTER COLUMN policy_id TYPE VARCHAR(64)
    USING ('quota_' || replace(policy_id::text, '-', ''));

ALTER TABLE quota.quota_usage
    ADD CONSTRAINT quota_usage_policy_id_fkey
    FOREIGN KEY (policy_id) REFERENCES quota.quota_policies(id) ON DELETE CASCADE;
