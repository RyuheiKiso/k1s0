-- event_monitor の全テーブルのテナント分離を元に戻す。
-- RLS ポリシー・インデックス・tenant_id カラムを削除し、name UNIQUE INDEX を復元する。

BEGIN;

SET LOCAL search_path TO event_monitor, public;

-- flow_instances の RLS 無効化とポリシー削除
DROP POLICY IF EXISTS tenant_isolation ON event_monitor.flow_instances;
ALTER TABLE event_monitor.flow_instances DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS event_monitor.idx_flow_instances_tenant_id;
ALTER TABLE event_monitor.flow_instances DROP COLUMN IF EXISTS tenant_id;

-- event_records の RLS 無効化とポリシー削除
DROP POLICY IF EXISTS tenant_isolation ON event_monitor.event_records;
ALTER TABLE event_monitor.event_records DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS event_monitor.idx_event_records_tenant_id;
ALTER TABLE event_monitor.event_records DROP COLUMN IF EXISTS tenant_id;

-- flow_definitions の RLS 無効化とポリシー削除
DROP POLICY IF EXISTS tenant_isolation ON event_monitor.flow_definitions;
ALTER TABLE event_monitor.flow_definitions DISABLE ROW LEVEL SECURITY;

-- テナントスコープ UNIQUE INDEX を削除し、元の name UNIQUE INDEX を復元する
DROP INDEX IF EXISTS event_monitor.idx_flow_definitions_tenant_name;
CREATE UNIQUE INDEX IF NOT EXISTS idx_flow_definitions_name
    ON event_monitor.flow_definitions (name);

DROP INDEX IF EXISTS event_monitor.idx_flow_definitions_tenant_id;
ALTER TABLE event_monitor.flow_definitions DROP COLUMN IF EXISTS tenant_id;

COMMIT;
