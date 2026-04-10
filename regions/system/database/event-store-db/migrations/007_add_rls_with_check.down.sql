-- WITH CHECK 付き RLS ポリシーを USING のみのポリシーに戻す（ロールバック用）
BEGIN;

-- event_streams テーブルの tenant_isolation ポリシーを USING のみで再作成する
DROP POLICY IF EXISTS tenant_isolation ON eventstore.event_streams;
CREATE POLICY tenant_isolation ON eventstore.event_streams
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

-- events テーブルの tenant_isolation ポリシーを USING のみで再作成する
DROP POLICY IF EXISTS tenant_isolation ON eventstore.events;
CREATE POLICY tenant_isolation ON eventstore.events
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

-- snapshots テーブルの tenant_isolation ポリシーを USING のみで再作成する
DROP POLICY IF EXISTS tenant_isolation ON eventstore.snapshots;
CREATE POLICY tenant_isolation ON eventstore.snapshots
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

COMMIT;
