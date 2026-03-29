-- 008_add_quota_usage_rls のロールバック
BEGIN;
DROP POLICY IF EXISTS tenant_isolation ON quota.quota_usage;
ALTER TABLE quota.quota_usage DISABLE ROW LEVEL SECURITY;
ALTER TABLE quota.quota_usage NO FORCE ROW LEVEL SECURITY;
COMMIT;
