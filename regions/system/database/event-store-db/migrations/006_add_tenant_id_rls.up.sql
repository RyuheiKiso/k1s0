-- event_streams と events テーブルに tenant_id を追加し、RLS でテナント分離を実現する。
-- 設計根拠: docs/architecture/multi-tenancy.md および ADR-0012 参照。
-- 既存データは tenant_id = 'system' でバックフィルし、その後 DEFAULT を削除して NOT NULL を維持する。
-- RLS ポリシーにより app.current_tenant_id セッション変数でテナントを分離する。
-- snapshots テーブルは event_streams に従属するため、親テーブルと同じ tenant_id を持つ。

BEGIN;

-- event_streams テーブルに tenant_id カラムを追加する（既存データのバックフィルとして 'system' をデフォルト値とする）
ALTER TABLE eventstore.event_streams
    ADD COLUMN IF NOT EXISTS tenant_id VARCHAR(255) NOT NULL DEFAULT 'system';

-- バックフィル後はデフォルト値を削除し、新規挿入時の明示指定を強制する
ALTER TABLE eventstore.event_streams
    ALTER COLUMN tenant_id DROP DEFAULT;

-- tenant_id のインデックスを追加してクエリ性能を確保する
CREATE INDEX IF NOT EXISTS idx_event_streams_tenant_id
    ON eventstore.event_streams (tenant_id);

-- テナントと集約タイプの複合インデックスを追加する（テナント横断クエリの高速化）
CREATE INDEX IF NOT EXISTS idx_event_streams_tenant_aggregate_type
    ON eventstore.event_streams (tenant_id, aggregate_type);

-- events テーブルに tenant_id カラムを追加する（親テーブル event_streams と整合性を保つ）
ALTER TABLE eventstore.events
    ADD COLUMN IF NOT EXISTS tenant_id VARCHAR(255) NOT NULL DEFAULT 'system';

-- バックフィル後はデフォルト値を削除する
ALTER TABLE eventstore.events
    ALTER COLUMN tenant_id DROP DEFAULT;

-- events の tenant_id インデックスを追加する
CREATE INDEX IF NOT EXISTS idx_events_tenant_id
    ON eventstore.events (tenant_id);

-- テナントとイベントタイプの複合インデックスを追加する（テナント別イベント検索の高速化）
CREATE INDEX IF NOT EXISTS idx_events_tenant_event_type
    ON eventstore.events (tenant_id, event_type);

-- snapshots テーブルに tenant_id カラムを追加する（event_streams と整合性を保つ）
ALTER TABLE eventstore.snapshots
    ADD COLUMN IF NOT EXISTS tenant_id VARCHAR(255) NOT NULL DEFAULT 'system';

-- バックフィル後はデフォルト値を削除する
ALTER TABLE eventstore.snapshots
    ALTER COLUMN tenant_id DROP DEFAULT;

-- snapshots の tenant_id インデックスを追加する
CREATE INDEX IF NOT EXISTS idx_snapshots_tenant_id
    ON eventstore.snapshots (tenant_id);

-- event_streams テーブルの RLS を有効化する
ALTER TABLE eventstore.event_streams ENABLE ROW LEVEL SECURITY;

-- テナント分離ポリシーを設定する（app.current_tenant_id セッション変数でフィルタリング）
-- current_setting の第 2 引数 true = 変数未設定時に NULL を返すことでエラーを回避する
DROP POLICY IF EXISTS tenant_isolation ON eventstore.event_streams;
CREATE POLICY tenant_isolation ON eventstore.event_streams
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);

-- events テーブルの RLS を有効化する
ALTER TABLE eventstore.events ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON eventstore.events;
CREATE POLICY tenant_isolation ON eventstore.events
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);

-- snapshots テーブルの RLS を有効化する
ALTER TABLE eventstore.snapshots ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON eventstore.snapshots;
CREATE POLICY tenant_isolation ON eventstore.snapshots
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);

-- スーパーユーザー・オーナーロールも RLS の適用対象とする（バイパスを防止）
ALTER TABLE eventstore.event_streams FORCE ROW LEVEL SECURITY;
ALTER TABLE eventstore.events FORCE ROW LEVEL SECURITY;
ALTER TABLE eventstore.snapshots FORCE ROW LEVEL SECURITY;

COMMIT;
