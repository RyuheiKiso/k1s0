-- policies / policy_bundles テーブルの tenant_id を VARCHAR(255) から TEXT 型に変更する。
-- CRITICAL-DB-002 監査対応: 全サービスで tenant_id を TEXT 型に統一する（ADR-0093）。
-- HIGH-DB-007: policies テーブルの UNIQUE(name) を UNIQUE(tenant_id, name) に変更してテナント間重複を許可する。
-- 既存の RLS ポリシーを DROP してから型変更し、AS RESTRICTIVE + WITH CHECK で再作成する。

BEGIN;

SET LOCAL search_path TO policy, public;

-- 既存の RLS ポリシーを先に削除する
DROP POLICY IF EXISTS tenant_isolation ON policy.policies;
DROP POLICY IF EXISTS tenant_isolation ON policy.policy_bundles;

-- policies テーブルの tenant_id を TEXT 型に変更する
ALTER TABLE policy.policies
    ALTER COLUMN tenant_id TYPE TEXT USING tenant_id::TEXT;

-- policy_bundles テーブルの tenant_id を TEXT 型に変更する
ALTER TABLE policy.policy_bundles
    ALTER COLUMN tenant_id TYPE TEXT USING tenant_id::TEXT;

-- policies テーブルの UNIQUE(name) を UNIQUE(tenant_id, name) に変更する（HIGH-DB-007 対応）
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'policies_name_key' AND conrelid = 'policy.policies'::regclass
    ) THEN
        ALTER TABLE policy.policies DROP CONSTRAINT policies_name_key;
    END IF;
END $$;
CREATE UNIQUE INDEX IF NOT EXISTS uq_policies_tenant_name
    ON policy.policies (tenant_id, name);

-- policies テーブルのポリシーを再作成する
CREATE POLICY tenant_isolation ON policy.policies
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- policy_bundles テーブルのポリシーを再作成する
CREATE POLICY tenant_isolation ON policy.policy_bundles
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

COMMIT;
