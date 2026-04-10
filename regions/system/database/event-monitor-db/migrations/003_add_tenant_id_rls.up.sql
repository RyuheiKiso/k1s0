-- event_monitor の全テーブルにテナント分離を実装する。
-- CRITICAL-DB-001 監査対応: マルチテナント環境でテナント間データ漏洩を防止する。
-- flow_definitions / event_records / flow_instances の 3 テーブルに tenant_id + RLS を追加する。
-- flow_definitions の UNIQUE INDEX (name) はテナントスコープ化して UNIQUE(tenant_id, name) に変更する。
-- 既存データは 'system' テナントとしてバックフィルし、新規挿入を強制する。

BEGIN;

SET LOCAL search_path TO event_monitor, public;

-- flow_definitions テーブルに tenant_id カラムを追加する
ALTER TABLE event_monitor.flow_definitions
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

-- バックフィル後はデフォルト値を削除し、新規挿入時の明示指定を強制する
ALTER TABLE event_monitor.flow_definitions
    ALTER COLUMN tenant_id DROP DEFAULT;

-- tenant_id のインデックスを追加してクエリ性能を確保する
CREATE INDEX IF NOT EXISTS idx_flow_definitions_tenant_id
    ON event_monitor.flow_definitions (tenant_id);

-- 既存の name 単独 UNIQUE INDEX を削除し、テナントスコープの UNIQUE INDEX に変更する
DROP INDEX IF EXISTS event_monitor.idx_flow_definitions_name;
CREATE UNIQUE INDEX IF NOT EXISTS idx_flow_definitions_tenant_name
    ON event_monitor.flow_definitions (tenant_id, name);

-- flow_definitions テーブルの RLS を有効化する
ALTER TABLE event_monitor.flow_definitions ENABLE ROW LEVEL SECURITY;
ALTER TABLE event_monitor.flow_definitions FORCE ROW LEVEL SECURITY;

-- テナント分離ポリシーを設定する（AS RESTRICTIVE + WITH CHECK で INSERT/UPDATE/SELECT を保護）
DROP POLICY IF EXISTS tenant_isolation ON event_monitor.flow_definitions;
CREATE POLICY tenant_isolation ON event_monitor.flow_definitions
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- event_records テーブルに tenant_id カラムを追加する
ALTER TABLE event_monitor.event_records
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

-- バックフィル後はデフォルト値を削除する
ALTER TABLE event_monitor.event_records
    ALTER COLUMN tenant_id DROP DEFAULT;

-- tenant_id のインデックスを追加する
CREATE INDEX IF NOT EXISTS idx_event_records_tenant_id
    ON event_monitor.event_records (tenant_id);

-- event_records テーブルの RLS を有効化する
ALTER TABLE event_monitor.event_records ENABLE ROW LEVEL SECURITY;
ALTER TABLE event_monitor.event_records FORCE ROW LEVEL SECURITY;

-- テナント分離ポリシーを設定する
DROP POLICY IF EXISTS tenant_isolation ON event_monitor.event_records;
CREATE POLICY tenant_isolation ON event_monitor.event_records
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- flow_instances テーブルに tenant_id カラムを追加する
ALTER TABLE event_monitor.flow_instances
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

-- バックフィル後はデフォルト値を削除する
ALTER TABLE event_monitor.flow_instances
    ALTER COLUMN tenant_id DROP DEFAULT;

-- tenant_id のインデックスを追加する
CREATE INDEX IF NOT EXISTS idx_flow_instances_tenant_id
    ON event_monitor.flow_instances (tenant_id);

-- flow_instances テーブルの RLS を有効化する
ALTER TABLE event_monitor.flow_instances ENABLE ROW LEVEL SECURITY;
ALTER TABLE event_monitor.flow_instances FORCE ROW LEVEL SECURITY;

-- テナント分離ポリシーを設定する
DROP POLICY IF EXISTS tenant_isolation ON event_monitor.flow_instances;
CREATE POLICY tenant_isolation ON event_monitor.flow_instances
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

COMMIT;
