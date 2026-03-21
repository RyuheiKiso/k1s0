-- 006_add_tenant_id_rls.up.sql のロールバック。
-- tenant_id カラム、インデックス、RLS ポリシーを削除し、マイグレーション前の状態に戻す。
-- 注意: ロールバック時に tenant_id に格納されたデータは失われる。実行前にバックアップを取得すること。

BEGIN;

-- FORCE ROW LEVEL SECURITY を解除する
ALTER TABLE eventstore.snapshots NO FORCE ROW LEVEL SECURITY;
ALTER TABLE eventstore.events NO FORCE ROW LEVEL SECURITY;
ALTER TABLE eventstore.event_streams NO FORCE ROW LEVEL SECURITY;

-- テナント分離ポリシーを削除する
DROP POLICY IF EXISTS tenant_isolation ON eventstore.snapshots;
DROP POLICY IF EXISTS tenant_isolation ON eventstore.events;
DROP POLICY IF EXISTS tenant_isolation ON eventstore.event_streams;

-- RLS を無効化する
ALTER TABLE eventstore.snapshots DISABLE ROW LEVEL SECURITY;
ALTER TABLE eventstore.events DISABLE ROW LEVEL SECURITY;
ALTER TABLE eventstore.event_streams DISABLE ROW LEVEL SECURITY;

-- インデックスを削除する
DROP INDEX IF EXISTS eventstore.idx_snapshots_tenant_id;
DROP INDEX IF EXISTS eventstore.idx_events_tenant_event_type;
DROP INDEX IF EXISTS eventstore.idx_events_tenant_id;
DROP INDEX IF EXISTS eventstore.idx_event_streams_tenant_aggregate_type;
DROP INDEX IF EXISTS eventstore.idx_event_streams_tenant_id;

-- tenant_id カラムを削除する
ALTER TABLE eventstore.snapshots DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE eventstore.events DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE eventstore.event_streams DROP COLUMN IF EXISTS tenant_id;

COMMIT;
