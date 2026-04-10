-- event_streams / events / snapshots の tenant_id を TEXT から VARCHAR(255) に戻す。

BEGIN;

SET LOCAL search_path TO eventstore, public;

DROP POLICY IF EXISTS tenant_isolation ON eventstore.event_streams;
DROP POLICY IF EXISTS tenant_isolation ON eventstore.events;
DROP POLICY IF EXISTS tenant_isolation ON eventstore.snapshots;

ALTER TABLE eventstore.event_streams
    ALTER COLUMN tenant_id TYPE VARCHAR(255) USING tenant_id::VARCHAR(255);

ALTER TABLE eventstore.events
    ALTER COLUMN tenant_id TYPE VARCHAR(255) USING tenant_id::VARCHAR(255);

ALTER TABLE eventstore.snapshots
    ALTER COLUMN tenant_id TYPE VARCHAR(255) USING tenant_id::VARCHAR(255);

-- 007 マイグレーション相当のポリシーを復元する
CREATE POLICY tenant_isolation ON eventstore.event_streams
    AS RESTRICTIVE
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT)
    WITH CHECK (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

CREATE POLICY tenant_isolation ON eventstore.events
    AS RESTRICTIVE
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT)
    WITH CHECK (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

CREATE POLICY tenant_isolation ON eventstore.snapshots
    AS RESTRICTIVE
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT)
    WITH CHECK (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

COMMIT;
