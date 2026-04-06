-- HIGH-009 監査対応ロールバック: outbox_events RLS を 004 時点の状態（WITH CHECK なし）に戻す。
-- 注意: ロールバック後はテナント間の INSERT 制御が無効になるため、本番環境でのロールバックは推奨しない。
SET LOCAL search_path TO board_service, public;

ALTER TABLE outbox_events NO FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON outbox_events;

CREATE POLICY tenant_isolation ON outbox_events
    USING (
        current_setting('app.current_tenant_id', true) IS NULL
        OR tenant_id = current_setting('app.current_tenant_id', true)::TEXT
    );
