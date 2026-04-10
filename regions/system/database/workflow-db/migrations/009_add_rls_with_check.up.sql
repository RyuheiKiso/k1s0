-- RLS ポリシーに WITH CHECK 句を追加して INSERT/UPDATE 時のテナント検証を有効化する
-- USING 句のみの場合、SELECT/UPDATE/DELETE はテナント分離されるが INSERT は検証されない（CWE-284）
-- AS RESTRICTIVE により他のポリシーが存在しても必ずこのポリシーで制限される
-- workflow_definitions・workflow_instances・workflow_tasks の 3 テーブルに適用する
-- SET LOCAL でトランザクションスコープに限定し、セッション汚染を防止する
SET LOCAL search_path TO workflow, public;

-- workflow_definitions テーブルの tenant_isolation ポリシーを WITH CHECK 付きで再作成する
DROP POLICY IF EXISTS tenant_isolation ON workflow.workflow_definitions;
CREATE POLICY tenant_isolation ON workflow.workflow_definitions
    AS RESTRICTIVE
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT)
    WITH CHECK (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

-- workflow_instances テーブルの tenant_isolation ポリシーを WITH CHECK 付きで再作成する
DROP POLICY IF EXISTS tenant_isolation ON workflow.workflow_instances;
CREATE POLICY tenant_isolation ON workflow.workflow_instances
    AS RESTRICTIVE
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT)
    WITH CHECK (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

-- workflow_tasks テーブルの tenant_isolation ポリシーを WITH CHECK 付きで再作成する
DROP POLICY IF EXISTS tenant_isolation ON workflow.workflow_tasks;
CREATE POLICY tenant_isolation ON workflow.workflow_tasks
    AS RESTRICTIVE
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT)
    WITH CHECK (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);
