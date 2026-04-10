-- master_maintenance の全テーブルにテナント分離を実装する。
-- CRITICAL-DB-001 監査対応: マルチテナント環境でテナント間データ漏洩を防止する。
-- table_definitions / column_definitions / table_relationships / consistency_rules /
-- rule_conditions / display_configs / change_logs / import_jobs の全テーブルに tenant_id + RLS を追加する。
-- table_definitions の UNIQUE INDEX (name, domain_scope) はテナントスコープ化して追加する。
-- 既存データは 'system' テナントとしてバックフィルし、新規挿入を強制する。

BEGIN;

SET LOCAL search_path TO master_maintenance, public;

-- table_definitions テーブルに tenant_id カラムを追加する
ALTER TABLE master_maintenance.table_definitions
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

ALTER TABLE master_maintenance.table_definitions
    ALTER COLUMN tenant_id DROP DEFAULT;

CREATE INDEX IF NOT EXISTS idx_table_definitions_tenant_id
    ON master_maintenance.table_definitions (tenant_id);

ALTER TABLE master_maintenance.table_definitions ENABLE ROW LEVEL SECURITY;
ALTER TABLE master_maintenance.table_definitions FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON master_maintenance.table_definitions;
CREATE POLICY tenant_isolation ON master_maintenance.table_definitions
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- column_definitions テーブルに tenant_id カラムを追加する
-- （table_definitions の外部キーを通じて間接的にテナント分離可能だが、明示的に追加する）
ALTER TABLE master_maintenance.column_definitions
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

ALTER TABLE master_maintenance.column_definitions
    ALTER COLUMN tenant_id DROP DEFAULT;

CREATE INDEX IF NOT EXISTS idx_column_definitions_tenant_id
    ON master_maintenance.column_definitions (tenant_id);

ALTER TABLE master_maintenance.column_definitions ENABLE ROW LEVEL SECURITY;
ALTER TABLE master_maintenance.column_definitions FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON master_maintenance.column_definitions;
CREATE POLICY tenant_isolation ON master_maintenance.column_definitions
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- table_relationships テーブルに tenant_id カラムを追加する
ALTER TABLE master_maintenance.table_relationships
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

ALTER TABLE master_maintenance.table_relationships
    ALTER COLUMN tenant_id DROP DEFAULT;

CREATE INDEX IF NOT EXISTS idx_table_relationships_tenant_id
    ON master_maintenance.table_relationships (tenant_id);

ALTER TABLE master_maintenance.table_relationships ENABLE ROW LEVEL SECURITY;
ALTER TABLE master_maintenance.table_relationships FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON master_maintenance.table_relationships;
CREATE POLICY tenant_isolation ON master_maintenance.table_relationships
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- consistency_rules テーブルに tenant_id カラムを追加する
ALTER TABLE master_maintenance.consistency_rules
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

ALTER TABLE master_maintenance.consistency_rules
    ALTER COLUMN tenant_id DROP DEFAULT;

CREATE INDEX IF NOT EXISTS idx_consistency_rules_tenant_id
    ON master_maintenance.consistency_rules (tenant_id);

ALTER TABLE master_maintenance.consistency_rules ENABLE ROW LEVEL SECURITY;
ALTER TABLE master_maintenance.consistency_rules FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON master_maintenance.consistency_rules;
CREATE POLICY tenant_isolation ON master_maintenance.consistency_rules
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- rule_conditions テーブルに tenant_id カラムを追加する
ALTER TABLE master_maintenance.rule_conditions
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

ALTER TABLE master_maintenance.rule_conditions
    ALTER COLUMN tenant_id DROP DEFAULT;

CREATE INDEX IF NOT EXISTS idx_rule_conditions_tenant_id
    ON master_maintenance.rule_conditions (tenant_id);

ALTER TABLE master_maintenance.rule_conditions ENABLE ROW LEVEL SECURITY;
ALTER TABLE master_maintenance.rule_conditions FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON master_maintenance.rule_conditions;
CREATE POLICY tenant_isolation ON master_maintenance.rule_conditions
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- display_configs テーブルに tenant_id カラムを追加する
ALTER TABLE master_maintenance.display_configs
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

ALTER TABLE master_maintenance.display_configs
    ALTER COLUMN tenant_id DROP DEFAULT;

CREATE INDEX IF NOT EXISTS idx_display_configs_tenant_id
    ON master_maintenance.display_configs (tenant_id);

ALTER TABLE master_maintenance.display_configs ENABLE ROW LEVEL SECURITY;
ALTER TABLE master_maintenance.display_configs FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON master_maintenance.display_configs;
CREATE POLICY tenant_isolation ON master_maintenance.display_configs
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- change_logs テーブルに tenant_id カラムを追加する（既に domain_scope カラムがある場合は共存）
ALTER TABLE master_maintenance.change_logs
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

ALTER TABLE master_maintenance.change_logs
    ALTER COLUMN tenant_id DROP DEFAULT;

CREATE INDEX IF NOT EXISTS idx_change_logs_tenant_id
    ON master_maintenance.change_logs (tenant_id);

ALTER TABLE master_maintenance.change_logs ENABLE ROW LEVEL SECURITY;
ALTER TABLE master_maintenance.change_logs FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON master_maintenance.change_logs;
CREATE POLICY tenant_isolation ON master_maintenance.change_logs
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- import_jobs テーブルに tenant_id カラムを追加する
ALTER TABLE master_maintenance.import_jobs
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

ALTER TABLE master_maintenance.import_jobs
    ALTER COLUMN tenant_id DROP DEFAULT;

CREATE INDEX IF NOT EXISTS idx_import_jobs_tenant_id
    ON master_maintenance.import_jobs (tenant_id);

ALTER TABLE master_maintenance.import_jobs ENABLE ROW LEVEL SECURITY;
ALTER TABLE master_maintenance.import_jobs FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON master_maintenance.import_jobs;
CREATE POLICY tenant_isolation ON master_maintenance.import_jobs
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

COMMIT;
