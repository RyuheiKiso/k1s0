DROP TRIGGER IF EXISTS trigger_workflow_instances_update_updated_at ON workflow.workflow_instances;
DROP INDEX IF EXISTS workflow.idx_workflow_instances_initiator_id;
DROP INDEX IF EXISTS workflow.idx_workflow_instances_status;
DROP INDEX IF EXISTS workflow.idx_workflow_instances_definition_id;
DROP TABLE IF EXISTS workflow.workflow_instances;
