-- workflow テーブルの FORCE ROW LEVEL SECURITY と UNIQUE 制約変更を元に戻す。

BEGIN;

SET LOCAL search_path TO workflow, public;

-- テナントスコープ UNIQUE INDEX を削除し、元の name インデックスを復元する
DROP INDEX IF EXISTS workflow.uq_workflow_definitions_tenant_name;
CREATE UNIQUE INDEX IF NOT EXISTS idx_workflow_definitions_name
    ON workflow.workflow_definitions (name);

-- FORCE ROW LEVEL SECURITY を解除する（NO FORCE に戻す）
ALTER TABLE workflow.workflow_tasks NO FORCE ROW LEVEL SECURITY;
ALTER TABLE workflow.workflow_instances NO FORCE ROW LEVEL SECURITY;
ALTER TABLE workflow.workflow_definitions NO FORCE ROW LEVEL SECURITY;

COMMIT;
