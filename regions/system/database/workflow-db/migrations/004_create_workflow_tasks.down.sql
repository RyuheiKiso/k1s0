DROP TRIGGER IF EXISTS trigger_workflow_tasks_update_updated_at ON workflow.workflow_tasks;
DROP INDEX IF EXISTS workflow.idx_workflow_tasks_due_at;
DROP INDEX IF EXISTS workflow.idx_workflow_tasks_status;
DROP INDEX IF EXISTS workflow.idx_workflow_tasks_assignee_id;
DROP INDEX IF EXISTS workflow.idx_workflow_tasks_instance_id;
DROP TABLE IF EXISTS workflow.workflow_tasks;
