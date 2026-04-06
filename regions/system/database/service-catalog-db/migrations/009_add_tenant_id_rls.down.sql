-- service_catalog の全テーブルのテナント分離を元に戻す。
-- RLS ポリシー・インデックス・tenant_id カラムを削除し、teams の UNIQUE 制約を復元する。

BEGIN;

SET LOCAL search_path TO service_catalog, public;

-- scorecards の RLS 無効化とポリシー削除
DROP POLICY IF EXISTS tenant_isolation ON service_catalog.scorecards;
ALTER TABLE service_catalog.scorecards DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS service_catalog.idx_scorecards_tenant_id;
ALTER TABLE service_catalog.scorecards DROP COLUMN IF EXISTS tenant_id;

-- service_docs の RLS 無効化とポリシー削除
DROP POLICY IF EXISTS tenant_isolation ON service_catalog.service_docs;
ALTER TABLE service_catalog.service_docs DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS service_catalog.idx_service_docs_tenant_id;
ALTER TABLE service_catalog.service_docs DROP COLUMN IF EXISTS tenant_id;

-- health_status の RLS 無効化とポリシー削除
DROP POLICY IF EXISTS tenant_isolation ON service_catalog.health_status;
ALTER TABLE service_catalog.health_status DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS service_catalog.idx_health_status_tenant_id;
ALTER TABLE service_catalog.health_status DROP COLUMN IF EXISTS tenant_id;

-- dependencies の RLS 無効化とポリシー削除
DROP POLICY IF EXISTS tenant_isolation ON service_catalog.dependencies;
ALTER TABLE service_catalog.dependencies DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS service_catalog.idx_dependencies_tenant_id;
ALTER TABLE service_catalog.dependencies DROP COLUMN IF EXISTS tenant_id;

-- services の RLS 無効化とポリシー削除
DROP POLICY IF EXISTS tenant_isolation ON service_catalog.services;
ALTER TABLE service_catalog.services DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS service_catalog.idx_services_tenant_id;
ALTER TABLE service_catalog.services DROP COLUMN IF EXISTS tenant_id;

-- teams の RLS 無効化とポリシー削除
DROP POLICY IF EXISTS tenant_isolation ON service_catalog.teams;
ALTER TABLE service_catalog.teams DISABLE ROW LEVEL SECURITY;

-- テナントスコープ UNIQUE INDEX を削除し、元の name UNIQUE 制約を復元する
DROP INDEX IF EXISTS service_catalog.uq_teams_tenant_name;
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'teams_name_key' AND conrelid = 'service_catalog.teams'::regclass
    ) THEN
        ALTER TABLE service_catalog.teams ADD CONSTRAINT teams_name_key UNIQUE (name);
    END IF;
END $$;

DROP INDEX IF EXISTS service_catalog.idx_teams_tenant_id;
ALTER TABLE service_catalog.teams DROP COLUMN IF EXISTS tenant_id;

COMMIT;
