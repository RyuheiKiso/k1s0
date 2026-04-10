-- event_streams / events / snapshots テーブルの tenant_id を VARCHAR(255) から TEXT 型に変更する。
-- CRITICAL-DB-002 監査対応: 全サービスで tenant_id を TEXT 型に統一する（ADR-0093）。
-- 既存の RLS ポリシーを DROP してから型変更し、AS RESTRICTIVE + WITH CHECK で再作成する。

BEGIN;

SET LOCAL search_path TO eventstore, public;

-- 既存の RLS ポリシーを先に削除する
DROP POLICY IF EXISTS tenant_isolation ON eventstore.event_streams;
DROP POLICY IF EXISTS tenant_isolation ON eventstore.events;
DROP POLICY IF EXISTS tenant_isolation ON eventstore.snapshots;

-- event_streams テーブルの tenant_id を TEXT 型に変更する
ALTER TABLE eventstore.event_streams
    ALTER COLUMN tenant_id TYPE TEXT USING tenant_id::TEXT;

-- events テーブルの tenant_id を TEXT 型に変更する
ALTER TABLE eventstore.events
    ALTER COLUMN tenant_id TYPE TEXT USING tenant_id::TEXT;

-- snapshots テーブルの tenant_id を TEXT 型に変更する
ALTER TABLE eventstore.snapshots
    ALTER COLUMN tenant_id TYPE TEXT USING tenant_id::TEXT;

-- event_streams テーブルのポリシーを再作成する
CREATE POLICY tenant_isolation ON eventstore.event_streams
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- events テーブルのポリシーを再作成する
CREATE POLICY tenant_isolation ON eventstore.events
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- snapshots テーブルのポリシーを再作成する
CREATE POLICY tenant_isolation ON eventstore.snapshots
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

COMMIT;
