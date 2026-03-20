-- workflow-db: ON DELETE CASCADE を元の制約（CASCADE なし）に戻す

ALTER TABLE workflow.workflow_instances
    DROP CONSTRAINT IF EXISTS workflow_instances_definition_id_fkey;

ALTER TABLE workflow.workflow_instances
    ADD CONSTRAINT workflow_instances_definition_id_fkey
        FOREIGN KEY (definition_id)
        REFERENCES workflow.workflow_definitions(id);
