-- RLS ポリシーに WITH CHECK 句を追加して INSERT/UPDATE 時のテナント検証を有効化する
-- USING 句のみの場合、SELECT/UPDATE/DELETE はテナント分離されるが INSERT は検証されない（CWE-284）
-- AS RESTRICTIVE により他のポリシーが存在しても必ずこのポリシーで制限される
-- event_streams・events・snapshots の 3 テーブルに適用する
BEGIN;

-- event_streams テーブルの tenant_isolation ポリシーを WITH CHECK 付きで再作成する
DROP POLICY IF EXISTS tenant_isolation ON eventstore.event_streams;
CREATE POLICY tenant_isolation ON eventstore.event_streams
    AS RESTRICTIVE
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT)
    WITH CHECK (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

-- events テーブルの tenant_isolation ポリシーを WITH CHECK 付きで再作成する
DROP POLICY IF EXISTS tenant_isolation ON eventstore.events;
CREATE POLICY tenant_isolation ON eventstore.events
    AS RESTRICTIVE
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT)
    WITH CHECK (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

-- snapshots テーブルの tenant_isolation ポリシーを WITH CHECK 付きで再作成する
DROP POLICY IF EXISTS tenant_isolation ON eventstore.snapshots;
CREATE POLICY tenant_isolation ON eventstore.snapshots
    AS RESTRICTIVE
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT)
    WITH CHECK (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

COMMIT;
