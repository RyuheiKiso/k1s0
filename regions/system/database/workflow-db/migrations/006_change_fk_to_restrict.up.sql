-- workflow-db: definition_id FK を ON DELETE CASCADE から ON DELETE RESTRICT に変更する
-- CASCADE では definition 削除時に全 instances が自動削除され、データ損失リスクがある。
-- RESTRICT に変更し、instances が存在する definition を誤って削除できないようにする。

-- 既存の CASCADE 制約を削除してから RESTRICT 付きで再作成する
ALTER TABLE workflow.workflow_instances
    DROP CONSTRAINT IF EXISTS workflow_instances_definition_id_fkey;

ALTER TABLE workflow.workflow_instances
    ADD CONSTRAINT workflow_instances_definition_id_fkey
        FOREIGN KEY (definition_id)
        REFERENCES workflow.workflow_definitions(id)
        ON DELETE RESTRICT;
