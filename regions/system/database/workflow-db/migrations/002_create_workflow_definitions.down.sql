DROP TRIGGER IF EXISTS trigger_workflow_definitions_update_updated_at ON workflow.workflow_definitions;
DROP INDEX IF EXISTS workflow.idx_workflow_definitions_enabled;
DROP INDEX IF EXISTS workflow.idx_workflow_definitions_name;
DROP TABLE IF EXISTS workflow.workflow_definitions;
