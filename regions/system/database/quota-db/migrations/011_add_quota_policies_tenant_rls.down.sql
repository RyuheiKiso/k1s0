-- quota.quota_policies のテナント分離を元に戻す。

BEGIN;

SET LOCAL search_path TO quota, public;

DROP POLICY IF EXISTS tenant_isolation ON quota.quota_policies;
ALTER TABLE quota.quota_policies DISABLE ROW LEVEL SECURITY;

-- テナントスコープ UNIQUE INDEX を削除し、元の name UNIQUE 制約を復元する
DROP INDEX IF EXISTS quota.uq_quota_policies_tenant_name;
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'quota_policies_name_key' AND conrelid = 'quota.quota_policies'::regclass
    ) THEN
        ALTER TABLE quota.quota_policies ADD CONSTRAINT quota_policies_name_key UNIQUE (name);
    END IF;
END $$;

DROP INDEX IF EXISTS quota.idx_quota_policies_tenant_id;
ALTER TABLE quota.quota_policies DROP COLUMN IF EXISTS tenant_id;

COMMIT;
