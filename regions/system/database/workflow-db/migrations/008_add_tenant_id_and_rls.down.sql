-- workflow テナント分離の逆マイグレーション
SET search_path TO workflow;

DROP POLICY IF EXISTS tenant_isolation ON workflow.workflow_tasks;
ALTER TABLE workflow.workflow_tasks DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS idx_workflow_tasks_tenant_id;
ALTER TABLE workflow.workflow_tasks DROP COLUMN IF EXISTS tenant_id;

DROP POLICY IF EXISTS tenant_isolation ON workflow.workflow_instances;
ALTER TABLE workflow.workflow_instances DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS idx_workflow_instances_tenant_id;
ALTER TABLE workflow.workflow_instances DROP COLUMN IF EXISTS tenant_id;

DROP POLICY IF EXISTS tenant_isolation ON workflow.workflow_definitions;
ALTER TABLE workflow.workflow_definitions DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS idx_workflow_definitions_tenant_id;
ALTER TABLE workflow.workflow_definitions DROP COLUMN IF EXISTS tenant_id;
