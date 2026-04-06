-- task_service の RLS ポリシーを 005 マイグレーション相当に戻す。

BEGIN;

SET LOCAL search_path TO task_service, public;

-- task_checklist_items のポリシーを 005 相当に戻す
DROP POLICY IF EXISTS tenant_isolation ON task_service.task_checklist_items;
CREATE POLICY tenant_isolation ON task_service.task_checklist_items
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);
ALTER TABLE task_service.task_checklist_items NO FORCE ROW LEVEL SECURITY;

-- tasks のポリシーを 005 相当に戻す
DROP POLICY IF EXISTS tenant_isolation ON task_service.tasks;
CREATE POLICY tenant_isolation ON task_service.tasks
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);
ALTER TABLE task_service.tasks NO FORCE ROW LEVEL SECURITY;

COMMIT;
