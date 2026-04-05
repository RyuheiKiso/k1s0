-- WITH CHECK 付き RLS ポリシーを USING のみのポリシーに戻す（ロールバック用）
-- SET LOCAL でトランザクションスコープに限定し、セッション汚染を防止する
SET LOCAL search_path TO workflow, public;

-- workflow_definitions テーブルの tenant_isolation ポリシーを USING のみで再作成する
DROP POLICY IF EXISTS tenant_isolation ON workflow.workflow_definitions;
CREATE POLICY tenant_isolation ON workflow.workflow_definitions
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

-- workflow_instances テーブルの tenant_isolation ポリシーを USING のみで再作成する
DROP POLICY IF EXISTS tenant_isolation ON workflow.workflow_instances;
CREATE POLICY tenant_isolation ON workflow.workflow_instances
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

-- workflow_tasks テーブルの tenant_isolation ポリシーを USING のみで再作成する
DROP POLICY IF EXISTS tenant_isolation ON workflow.workflow_tasks;
CREATE POLICY tenant_isolation ON workflow.workflow_tasks
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);
