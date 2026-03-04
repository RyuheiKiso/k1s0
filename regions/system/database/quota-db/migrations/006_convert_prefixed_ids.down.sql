ALTER TABLE quota.quota_usage
    DROP CONSTRAINT IF EXISTS quota_usage_policy_id_fkey;

ALTER TABLE quota.quota_usage
    ALTER COLUMN policy_id TYPE UUID
    USING regexp_replace(substr(policy_id, 7), '(.{8})(.{4})(.{4})(.{4})(.{12})', '\\1-\\2-\\3-\\4-\\5')::uuid;

ALTER TABLE quota.quota_policies
    ALTER COLUMN id TYPE UUID
    USING regexp_replace(substr(id, 7), '(.{8})(.{4})(.{4})(.{4})(.{12})', '\\1-\\2-\\3-\\4-\\5')::uuid,
    ALTER COLUMN id SET DEFAULT gen_random_uuid();

ALTER TABLE quota.quota_usage
    ADD CONSTRAINT quota_usage_policy_id_fkey
    FOREIGN KEY (policy_id) REFERENCES quota.quota_policies(id) ON DELETE CASCADE;
