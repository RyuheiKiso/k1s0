-- tenant_project_extensions テーブルの RLS を無効化し、マイグレーション前の状態に戻す。
BEGIN;

DROP POLICY IF EXISTS tenant_isolation ON project_master.tenant_project_extensions;
DROP INDEX IF EXISTS project_master.idx_tenant_project_extensions_rls;
ALTER TABLE project_master.tenant_project_extensions DISABLE ROW LEVEL SECURITY;

COMMIT;
