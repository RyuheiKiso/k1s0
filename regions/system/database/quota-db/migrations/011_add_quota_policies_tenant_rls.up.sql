-- quota.quota_policies テーブルにテナント分離を実装する。
-- HIGH-DB-001 監査対応: quota_usage は 008/010 で対応済みだが、quota_policies は未対応。
-- HIGH-DB-007: UNIQUE(name) を UNIQUE(tenant_id, name) に変更してテナント間重複を許可する。
-- 既存データは 'system' テナントとしてバックフィルし、新規挿入を強制する。

BEGIN;

SET LOCAL search_path TO quota, public;

-- quota_policies テーブルに tenant_id カラムを追加する
ALTER TABLE quota.quota_policies
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

-- バックフィル後はデフォルト値を削除し、新規挿入時の明示指定を強制する
ALTER TABLE quota.quota_policies
    ALTER COLUMN tenant_id DROP DEFAULT;

-- tenant_id のインデックスを追加する
CREATE INDEX IF NOT EXISTS idx_quota_policies_tenant_id
    ON quota.quota_policies (tenant_id);

-- 既存の UNIQUE(name) 制約を削除し、テナントスコープの UNIQUE に変更する（HIGH-DB-007 対応）
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'quota_policies_name_key' AND conrelid = 'quota.quota_policies'::regclass
    ) THEN
        ALTER TABLE quota.quota_policies DROP CONSTRAINT quota_policies_name_key;
    END IF;
END $$;
CREATE UNIQUE INDEX IF NOT EXISTS uq_quota_policies_tenant_name
    ON quota.quota_policies (tenant_id, name);

-- quota_policies テーブルの RLS を有効化する
ALTER TABLE quota.quota_policies ENABLE ROW LEVEL SECURITY;
ALTER TABLE quota.quota_policies FORCE ROW LEVEL SECURITY;

-- テナント分離ポリシーを設定する
DROP POLICY IF EXISTS tenant_isolation ON quota.quota_policies;
CREATE POLICY tenant_isolation ON quota.quota_policies
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

COMMIT;
