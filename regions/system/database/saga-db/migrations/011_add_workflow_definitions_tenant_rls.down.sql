-- saga.workflow_definitions のテナント分離を元に戻す。

BEGIN;

SET LOCAL search_path TO saga, public;

DROP POLICY IF EXISTS tenant_isolation ON saga.workflow_definitions;
ALTER TABLE saga.workflow_definitions DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS saga.idx_workflow_definitions_tenant_id;
ALTER TABLE saga.workflow_definitions DROP COLUMN IF EXISTS tenant_id;

COMMIT;
