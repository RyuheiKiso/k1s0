DROP TRIGGER IF EXISTS update_workflow_tasks_updated_at ON workflow.workflow_tasks;
DROP TRIGGER IF EXISTS update_workflow_definitions_updated_at ON workflow.workflow_definitions;

DROP TABLE IF EXISTS workflow.workflow_tasks;
DROP TABLE IF EXISTS workflow.workflow_instances;
DROP TABLE IF EXISTS workflow.workflow_definitions;

DROP SCHEMA IF EXISTS workflow;
