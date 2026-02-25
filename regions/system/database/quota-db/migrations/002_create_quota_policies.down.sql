DROP TRIGGER IF EXISTS trigger_quota_policies_update_updated_at ON quota.quota_policies;
DROP INDEX IF EXISTS quota.idx_quota_policies_subject;
DROP INDEX IF EXISTS quota.idx_quota_policies_name;
DROP TABLE IF EXISTS quota.quota_policies;
