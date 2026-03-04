DROP TRIGGER IF EXISTS update_workflow_definitions_updated_at ON saga.workflow_definitions;
DROP INDEX IF EXISTS saga.idx_workflow_definitions_enabled;
DROP TABLE IF EXISTS saga.workflow_definitions;
