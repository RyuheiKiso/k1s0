-- tenant_project_extensions テーブルにテナント分離のための RLS を有効化する。
-- docs/architecture/multi-tenancy.md の Phase 3 対応。
-- tenant_id NOT NULL 制約が既に存在するため、カラム追加は不要。
BEGIN;

-- RLS を有効化し、スーパーユーザーによるバイパスも防止する
ALTER TABLE project_master.tenant_project_extensions ENABLE ROW LEVEL SECURITY;
ALTER TABLE project_master.tenant_project_extensions FORCE ROW LEVEL SECURITY;

-- 既存ポリシーを削除（冪等性のため）し、テナント分離ポリシーを再作成する
DROP POLICY IF EXISTS tenant_isolation ON project_master.tenant_project_extensions;
CREATE POLICY tenant_isolation ON project_master.tenant_project_extensions
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);

-- tenant_id による検索を高速化するためのインデックスを追加する
CREATE INDEX IF NOT EXISTS idx_tenant_project_extensions_rls
    ON project_master.tenant_project_extensions (tenant_id);

COMMIT;
