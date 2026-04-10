-- task_service の RLS ポリシーに FORCE + AS RESTRICTIVE + WITH CHECK を追加する。
-- HIGH-BIZ-005 監査対応: 005 マイグレーションで RLS は設定されているが、
-- FORCE ROW LEVEL SECURITY・AS RESTRICTIVE・WITH CHECK が欠落していた。
-- tasks と task_checklist_items の両テーブルに適用する。

BEGIN;

SET LOCAL search_path TO task_service, public;

-- tasks テーブルに FORCE ROW LEVEL SECURITY を追加する
ALTER TABLE task_service.tasks FORCE ROW LEVEL SECURITY;

-- tasks の tenant_isolation ポリシーを AS RESTRICTIVE + WITH CHECK 付きで再作成する
DROP POLICY IF EXISTS tenant_isolation ON task_service.tasks;
CREATE POLICY tenant_isolation ON task_service.tasks
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- task_checklist_items テーブルに FORCE ROW LEVEL SECURITY を追加する
ALTER TABLE task_service.task_checklist_items FORCE ROW LEVEL SECURITY;

-- task_checklist_items の tenant_isolation ポリシーを AS RESTRICTIVE + WITH CHECK 付きで再作成する
DROP POLICY IF EXISTS tenant_isolation ON task_service.task_checklist_items;
CREATE POLICY tenant_isolation ON task_service.task_checklist_items
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

COMMIT;
