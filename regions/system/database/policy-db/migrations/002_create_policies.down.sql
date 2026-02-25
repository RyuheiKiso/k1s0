DROP TRIGGER IF EXISTS trigger_policies_update_updated_at ON policy.policies;
DROP INDEX IF EXISTS policy.idx_policies_enabled;
DROP INDEX IF EXISTS policy.idx_policies_name;
DROP TABLE IF EXISTS policy.policies;
