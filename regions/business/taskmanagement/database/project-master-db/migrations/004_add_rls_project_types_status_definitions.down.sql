-- project_types および status_definitions テーブルの RLS と tenant_id カラムを削除して
-- マイグレーション前の状態に戻す。
BEGIN;

-- project_types テーブルのポリシーと RLS を削除する
DROP POLICY IF EXISTS tenant_isolation ON project_master.project_types;
DROP INDEX IF EXISTS project_master.idx_project_types_tenant_id;
ALTER TABLE project_master.project_types DISABLE ROW LEVEL SECURITY;
ALTER TABLE project_master.project_types DROP COLUMN IF EXISTS tenant_id;

-- status_definitions テーブルのポリシーと RLS を削除する
DROP POLICY IF EXISTS tenant_isolation ON project_master.status_definitions;
DROP INDEX IF EXISTS project_master.idx_status_definitions_tenant_id;
ALTER TABLE project_master.status_definitions DISABLE ROW LEVEL SECURITY;
ALTER TABLE project_master.status_definitions DROP COLUMN IF EXISTS tenant_id;

COMMIT;
