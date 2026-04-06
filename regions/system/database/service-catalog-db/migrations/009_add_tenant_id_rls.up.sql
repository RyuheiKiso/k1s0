-- service_catalog の全テーブルにテナント分離を実装する。
-- CRITICAL-DB-001 監査対応: マルチテナント環境でテナント間データ漏洩を防止する。
-- teams / services / dependencies / health_status / service_docs / scorecards の全テーブルに tenant_id + RLS を追加する。
-- teams テーブルの UNIQUE(name) は UNIQUE(tenant_id, name) に変更して、テナント間の名前重複を許可する。
-- 既存データは 'system' テナントとしてバックフィルし、新規挿入を強制する。

BEGIN;

SET LOCAL search_path TO service_catalog, public;

-- teams テーブルに tenant_id カラムを追加する
ALTER TABLE service_catalog.teams
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

ALTER TABLE service_catalog.teams
    ALTER COLUMN tenant_id DROP DEFAULT;

CREATE INDEX IF NOT EXISTS idx_teams_tenant_id
    ON service_catalog.teams (tenant_id);

-- 既存の name UNIQUE 制約を削除し、テナントスコープの UNIQUE 制約に変更する
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'teams_name_key' AND conrelid = 'service_catalog.teams'::regclass
    ) THEN
        ALTER TABLE service_catalog.teams DROP CONSTRAINT teams_name_key;
    END IF;
END $$;
CREATE UNIQUE INDEX IF NOT EXISTS uq_teams_tenant_name
    ON service_catalog.teams (tenant_id, name);

ALTER TABLE service_catalog.teams ENABLE ROW LEVEL SECURITY;
ALTER TABLE service_catalog.teams FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON service_catalog.teams;
CREATE POLICY tenant_isolation ON service_catalog.teams
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- services テーブルに tenant_id カラムを追加する
ALTER TABLE service_catalog.services
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

ALTER TABLE service_catalog.services
    ALTER COLUMN tenant_id DROP DEFAULT;

CREATE INDEX IF NOT EXISTS idx_services_tenant_id
    ON service_catalog.services (tenant_id);

ALTER TABLE service_catalog.services ENABLE ROW LEVEL SECURITY;
ALTER TABLE service_catalog.services FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON service_catalog.services;
CREATE POLICY tenant_isolation ON service_catalog.services
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- dependencies テーブルに tenant_id カラムを追加する
ALTER TABLE service_catalog.dependencies
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

ALTER TABLE service_catalog.dependencies
    ALTER COLUMN tenant_id DROP DEFAULT;

CREATE INDEX IF NOT EXISTS idx_dependencies_tenant_id
    ON service_catalog.dependencies (tenant_id);

ALTER TABLE service_catalog.dependencies ENABLE ROW LEVEL SECURITY;
ALTER TABLE service_catalog.dependencies FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON service_catalog.dependencies;
CREATE POLICY tenant_isolation ON service_catalog.dependencies
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- health_status テーブルに tenant_id カラムを追加する
ALTER TABLE service_catalog.health_status
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

ALTER TABLE service_catalog.health_status
    ALTER COLUMN tenant_id DROP DEFAULT;

CREATE INDEX IF NOT EXISTS idx_health_status_tenant_id
    ON service_catalog.health_status (tenant_id);

ALTER TABLE service_catalog.health_status ENABLE ROW LEVEL SECURITY;
ALTER TABLE service_catalog.health_status FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON service_catalog.health_status;
CREATE POLICY tenant_isolation ON service_catalog.health_status
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- service_docs テーブルに tenant_id カラムを追加する
ALTER TABLE service_catalog.service_docs
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

ALTER TABLE service_catalog.service_docs
    ALTER COLUMN tenant_id DROP DEFAULT;

CREATE INDEX IF NOT EXISTS idx_service_docs_tenant_id
    ON service_catalog.service_docs (tenant_id);

ALTER TABLE service_catalog.service_docs ENABLE ROW LEVEL SECURITY;
ALTER TABLE service_catalog.service_docs FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON service_catalog.service_docs;
CREATE POLICY tenant_isolation ON service_catalog.service_docs
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- scorecards テーブルに tenant_id カラムを追加する
ALTER TABLE service_catalog.scorecards
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

ALTER TABLE service_catalog.scorecards
    ALTER COLUMN tenant_id DROP DEFAULT;

CREATE INDEX IF NOT EXISTS idx_scorecards_tenant_id
    ON service_catalog.scorecards (tenant_id);

ALTER TABLE service_catalog.scorecards ENABLE ROW LEVEL SECURITY;
ALTER TABLE service_catalog.scorecards FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON service_catalog.scorecards;
CREATE POLICY tenant_isolation ON service_catalog.scorecards
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

COMMIT;
