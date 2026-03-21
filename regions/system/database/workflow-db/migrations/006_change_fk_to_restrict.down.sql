-- workflow-db: ON DELETE RESTRICT を ON DELETE CASCADE に戻す（ロールバック用）

ALTER TABLE workflow.workflow_instances
    DROP CONSTRAINT IF EXISTS workflow_instances_definition_id_fkey;

ALTER TABLE workflow.workflow_instances
    ADD CONSTRAINT workflow_instances_definition_id_fkey
        FOREIGN KEY (definition_id)
        REFERENCES workflow.workflow_definitions(id)
        ON DELETE CASCADE;
