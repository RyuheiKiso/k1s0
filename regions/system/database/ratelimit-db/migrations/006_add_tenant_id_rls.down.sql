-- 006_add_tenant_id_rls.up.sql のロールバック: RLS ポリシーと tenant_id カラムを削除する

BEGIN;

-- rate_limit_rules の RLS を無効化する
ALTER TABLE ratelimit.rate_limit_rules DISABLE ROW LEVEL SECURITY;
ALTER TABLE ratelimit.rate_limit_rules NO FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS tenant_isolation ON ratelimit.rate_limit_rules;

-- インデックスを削除する
DROP INDEX IF EXISTS ratelimit.idx_rate_limit_rules_tenant_id;
DROP INDEX IF EXISTS ratelimit.idx_rate_limit_rules_tenant_scope;

-- tenant_id カラムを削除する
ALTER TABLE ratelimit.rate_limit_rules DROP COLUMN IF EXISTS tenant_id;

COMMIT;
