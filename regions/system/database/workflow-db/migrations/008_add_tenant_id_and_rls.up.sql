-- workflow テナント分離: 全テーブルに tenant_id カラムと RLS ポリシーを追加する
-- RUST-CRIT-001 対応: テナント間のデータ漏洩を防止する
-- CRIT-002 監査対応: SET LOCAL でトランザクションスコープに限定し、セッション汚染を防止する
-- SET search_path（セッションレベル）は sqlx の _sqlx_migrations テーブル（public スキーマ）を
-- 見つけられなくするため、SET LOCAL + public を含む形に修正する
SET LOCAL search_path TO workflow, public;

-- workflow_definitions テーブルへの tenant_id 追加
ALTER TABLE workflow.workflow_definitions
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

CREATE INDEX IF NOT EXISTS idx_workflow_definitions_tenant_id
    ON workflow.workflow_definitions (tenant_id);

ALTER TABLE workflow.workflow_definitions ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON workflow.workflow_definitions;
CREATE POLICY tenant_isolation ON workflow.workflow_definitions
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);

-- workflow_instances テーブルへの tenant_id 追加
ALTER TABLE workflow.workflow_instances
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

CREATE INDEX IF NOT EXISTS idx_workflow_instances_tenant_id
    ON workflow.workflow_instances (tenant_id);

ALTER TABLE workflow.workflow_instances ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON workflow.workflow_instances;
CREATE POLICY tenant_isolation ON workflow.workflow_instances
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);

-- workflow_tasks テーブルへの tenant_id 追加（instances の外部キー経由でテナント分離可能だが明示的に追加）
ALTER TABLE workflow.workflow_tasks
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

CREATE INDEX IF NOT EXISTS idx_workflow_tasks_tenant_id
    ON workflow.workflow_tasks (tenant_id);

ALTER TABLE workflow.workflow_tasks ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON workflow.workflow_tasks;
CREATE POLICY tenant_isolation ON workflow.workflow_tasks
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);
