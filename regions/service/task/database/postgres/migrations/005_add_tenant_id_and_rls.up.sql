-- マルチテナント対応: tasks / task_checklist_items テーブルに tenant_id カラムを追加し、RLS ポリシーを設定する。
-- 設計根拠: docs/architecture/multi-tenancy.md Phase 1 対応。
-- 既存データは tenant_id = 'system' でバックフィルし、NOT NULL 制約を維持する。
-- RLS ポリシーにより app.current_tenant_id セッション変数でテナントを分離する。

SET search_path TO task_service;

ALTER TABLE tasks
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

ALTER TABLE task_checklist_items
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

CREATE INDEX IF NOT EXISTS idx_tasks_tenant_id ON tasks (tenant_id);
CREATE INDEX IF NOT EXISTS idx_tasks_tenant_project ON tasks (tenant_id, project_id);
CREATE INDEX IF NOT EXISTS idx_task_checklist_items_tenant_id ON task_checklist_items (tenant_id);

ALTER TABLE tasks ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON tasks;
CREATE POLICY tenant_isolation ON tasks
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);

ALTER TABLE task_checklist_items ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON task_checklist_items;
CREATE POLICY tenant_isolation ON task_checklist_items
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);
