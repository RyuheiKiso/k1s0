-- 005_add_tenant_id_rls.up.sql のロールバック: RLS ポリシーと tenant_id カラムを削除する

BEGIN;

-- policies の RLS を無効化する
ALTER TABLE policy.policies DISABLE ROW LEVEL SECURITY;
ALTER TABLE policy.policies NO FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS tenant_isolation ON policy.policies;

-- policy_bundles の RLS を無効化する
ALTER TABLE policy.policy_bundles DISABLE ROW LEVEL SECURITY;
ALTER TABLE policy.policy_bundles NO FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS tenant_isolation ON policy.policy_bundles;

-- インデックスを削除する
DROP INDEX IF EXISTS policy.idx_policies_tenant_id;
DROP INDEX IF EXISTS policy.idx_policies_tenant_enabled;
DROP INDEX IF EXISTS policy.idx_policy_bundles_tenant_id;

-- tenant_id カラムを削除する
ALTER TABLE policy.policies DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE policy.policy_bundles DROP COLUMN IF EXISTS tenant_id;

COMMIT;
