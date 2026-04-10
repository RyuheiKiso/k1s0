-- rate_limit_rules の tenant_id を TEXT から VARCHAR(255) に戻す。

BEGIN;

SET LOCAL search_path TO ratelimit, public;

DROP POLICY IF EXISTS tenant_isolation ON ratelimit.rate_limit_rules;

-- テナントスコープ UNIQUE INDEX を削除し、元の name UNIQUE 制約を復元する
DROP INDEX IF EXISTS ratelimit.uq_rate_limit_rules_tenant_name;
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'rate_limit_rules_name_key' AND conrelid = 'ratelimit.rate_limit_rules'::regclass
    ) THEN
        ALTER TABLE ratelimit.rate_limit_rules ADD CONSTRAINT rate_limit_rules_name_key UNIQUE (name);
    END IF;
END $$;

ALTER TABLE ratelimit.rate_limit_rules
    ALTER COLUMN tenant_id TYPE VARCHAR(255) USING tenant_id::VARCHAR(255);

-- 007 マイグレーション相当のポリシーを復元する
CREATE POLICY tenant_isolation ON ratelimit.rate_limit_rules
    AS RESTRICTIVE
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT)
    WITH CHECK (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

COMMIT;
