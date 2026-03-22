SET search_path TO task_service;
ALTER TABLE task_checklist_items DISABLE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS tenant_isolation ON task_checklist_items;
ALTER TABLE tasks DISABLE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS tenant_isolation ON tasks;
ALTER TABLE tasks DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE task_checklist_items DROP COLUMN IF EXISTS tenant_id;
