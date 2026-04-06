-- policies / policy_bundles の tenant_id を TEXT から VARCHAR(255) に戻す。

BEGIN;

SET LOCAL search_path TO policy, public;

DROP POLICY IF EXISTS tenant_isolation ON policy.policies;
DROP POLICY IF EXISTS tenant_isolation ON policy.policy_bundles;

-- テナントスコープ UNIQUE INDEX を削除し、元の name UNIQUE 制約を復元する
DROP INDEX IF EXISTS policy.uq_policies_tenant_name;
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'policies_name_key' AND conrelid = 'policy.policies'::regclass
    ) THEN
        ALTER TABLE policy.policies ADD CONSTRAINT policies_name_key UNIQUE (name);
    END IF;
END $$;

ALTER TABLE policy.policies
    ALTER COLUMN tenant_id TYPE VARCHAR(255) USING tenant_id::VARCHAR(255);

ALTER TABLE policy.policy_bundles
    ALTER COLUMN tenant_id TYPE VARCHAR(255) USING tenant_id::VARCHAR(255);

-- 006 マイグレーション相当のポリシーを復元する
CREATE POLICY tenant_isolation ON policy.policies
    AS RESTRICTIVE
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT)
    WITH CHECK (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

CREATE POLICY tenant_isolation ON policy.policy_bundles
    AS RESTRICTIVE
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT)
    WITH CHECK (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

COMMIT;
