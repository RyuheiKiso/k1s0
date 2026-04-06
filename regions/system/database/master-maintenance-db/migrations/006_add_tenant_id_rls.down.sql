-- master_maintenance の全テーブルのテナント分離を元に戻す。
-- RLS ポリシー・インデックス・tenant_id カラムを削除する。

BEGIN;

SET LOCAL search_path TO master_maintenance, public;

-- import_jobs の RLS 無効化とポリシー削除
DROP POLICY IF EXISTS tenant_isolation ON master_maintenance.import_jobs;
ALTER TABLE master_maintenance.import_jobs DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS master_maintenance.idx_import_jobs_tenant_id;
ALTER TABLE master_maintenance.import_jobs DROP COLUMN IF EXISTS tenant_id;

-- change_logs の RLS 無効化とポリシー削除
DROP POLICY IF EXISTS tenant_isolation ON master_maintenance.change_logs;
ALTER TABLE master_maintenance.change_logs DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS master_maintenance.idx_change_logs_tenant_id;
ALTER TABLE master_maintenance.change_logs DROP COLUMN IF EXISTS tenant_id;

-- display_configs の RLS 無効化とポリシー削除
DROP POLICY IF EXISTS tenant_isolation ON master_maintenance.display_configs;
ALTER TABLE master_maintenance.display_configs DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS master_maintenance.idx_display_configs_tenant_id;
ALTER TABLE master_maintenance.display_configs DROP COLUMN IF EXISTS tenant_id;

-- rule_conditions の RLS 無効化とポリシー削除
DROP POLICY IF EXISTS tenant_isolation ON master_maintenance.rule_conditions;
ALTER TABLE master_maintenance.rule_conditions DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS master_maintenance.idx_rule_conditions_tenant_id;
ALTER TABLE master_maintenance.rule_conditions DROP COLUMN IF EXISTS tenant_id;

-- consistency_rules の RLS 無効化とポリシー削除
DROP POLICY IF EXISTS tenant_isolation ON master_maintenance.consistency_rules;
ALTER TABLE master_maintenance.consistency_rules DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS master_maintenance.idx_consistency_rules_tenant_id;
ALTER TABLE master_maintenance.consistency_rules DROP COLUMN IF EXISTS tenant_id;

-- table_relationships の RLS 無効化とポリシー削除
DROP POLICY IF EXISTS tenant_isolation ON master_maintenance.table_relationships;
ALTER TABLE master_maintenance.table_relationships DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS master_maintenance.idx_table_relationships_tenant_id;
ALTER TABLE master_maintenance.table_relationships DROP COLUMN IF EXISTS tenant_id;

-- column_definitions の RLS 無効化とポリシー削除
DROP POLICY IF EXISTS tenant_isolation ON master_maintenance.column_definitions;
ALTER TABLE master_maintenance.column_definitions DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS master_maintenance.idx_column_definitions_tenant_id;
ALTER TABLE master_maintenance.column_definitions DROP COLUMN IF EXISTS tenant_id;

-- table_definitions の RLS 無効化とポリシー削除
DROP POLICY IF EXISTS tenant_isolation ON master_maintenance.table_definitions;
ALTER TABLE master_maintenance.table_definitions DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS master_maintenance.idx_table_definitions_tenant_id;
ALTER TABLE master_maintenance.table_definitions DROP COLUMN IF EXISTS tenant_id;

COMMIT;
