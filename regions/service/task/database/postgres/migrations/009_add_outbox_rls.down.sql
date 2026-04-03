-- HIGH-005 監査対応 ロールバック: outbox_events の RLS ポリシーと tenant_id カラムを削除する。
SET LOCAL search_path TO task_service, public;

DROP POLICY IF EXISTS tenant_isolation ON outbox_events;
ALTER TABLE outbox_events DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS idx_outbox_events_tenant_id;
ALTER TABLE outbox_events DROP COLUMN IF EXISTS tenant_id;
