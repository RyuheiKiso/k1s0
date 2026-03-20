-- workflow-db: workflow_instances の definition_id FK に ON DELETE CASCADE を追加する
-- definition 削除時に関連する instances が自動削除され、孤立レコードを防ぐ

-- 既存の外部キー制約を削除してから ON DELETE CASCADE 付きで再作成する
ALTER TABLE workflow.workflow_instances
    DROP CONSTRAINT IF EXISTS workflow_instances_definition_id_fkey;

ALTER TABLE workflow.workflow_instances
    ADD CONSTRAINT workflow_instances_definition_id_fkey
        FOREIGN KEY (definition_id)
        REFERENCES workflow.workflow_definitions(id)
        ON DELETE CASCADE;
