-- rate_limit_rules テーブルの tenant_id を VARCHAR(255) から TEXT 型に変更する。
-- CRITICAL-DB-002 監査対応: 全サービスで tenant_id を TEXT 型に統一する（ADR-0093）。
-- HIGH-DB-007: UNIQUE(name) を UNIQUE(tenant_id, name) に変更してテナント間重複を許可する。
-- 既存の RLS ポリシーを DROP してから型変更し、AS RESTRICTIVE + WITH CHECK で再作成する。

BEGIN;

SET LOCAL search_path TO ratelimit, public;

-- 既存の RLS ポリシーを先に削除する
DROP POLICY IF EXISTS tenant_isolation ON ratelimit.rate_limit_rules;

-- rate_limit_rules テーブルの tenant_id を TEXT 型に変更する
ALTER TABLE ratelimit.rate_limit_rules
    ALTER COLUMN tenant_id TYPE TEXT USING tenant_id::TEXT;

-- rate_limit_rules テーブルの UNIQUE(name) を UNIQUE(tenant_id, name) に変更する（HIGH-DB-007 対応）
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'rate_limit_rules_name_key' AND conrelid = 'ratelimit.rate_limit_rules'::regclass
    ) THEN
        ALTER TABLE ratelimit.rate_limit_rules DROP CONSTRAINT rate_limit_rules_name_key;
    END IF;
END $$;
CREATE UNIQUE INDEX IF NOT EXISTS uq_rate_limit_rules_tenant_name
    ON ratelimit.rate_limit_rules (tenant_id, name);

-- RLS ポリシーを再作成する
CREATE POLICY tenant_isolation ON ratelimit.rate_limit_rules
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

COMMIT;
