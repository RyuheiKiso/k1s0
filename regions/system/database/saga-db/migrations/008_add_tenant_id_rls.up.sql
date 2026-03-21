-- saga_states と saga_step_logs に tenant_id を追加し、RLS でテナント分離を実現する。
-- 設計根拠: docs/architecture/multi-tenancy.md および ADR-0012 参照。
-- 既存データは tenant_id = 'system' でバックフィルし、その後 DEFAULT を削除して NOT NULL を維持する。
-- RLS ポリシーにより app.current_tenant_id セッション変数でテナントを分離する。

BEGIN;

-- saga_states テーブルに tenant_id カラムを追加する（既存データのバックフィルとして 'system' をデフォルト値とする）
ALTER TABLE saga.saga_states
    ADD COLUMN IF NOT EXISTS tenant_id VARCHAR(255) NOT NULL DEFAULT 'system';

-- バックフィル後はデフォルト値を削除し、新規挿入時の明示指定を強制する
ALTER TABLE saga.saga_states
    ALTER COLUMN tenant_id DROP DEFAULT;

-- tenant_id のインデックスを追加してクエリ性能を確保する
CREATE INDEX IF NOT EXISTS idx_saga_states_tenant_id
    ON saga.saga_states (tenant_id);

-- テナントと相関 ID の複合インデックスを追加する（テナント横断クエリの高速化）
CREATE INDEX IF NOT EXISTS idx_saga_states_tenant_workflow
    ON saga.saga_states (tenant_id, workflow_name);

-- saga_step_logs テーブルに tenant_id カラムを追加する（親テーブル saga_states と整合性を保つ）
ALTER TABLE saga.saga_step_logs
    ADD COLUMN IF NOT EXISTS tenant_id VARCHAR(255) NOT NULL DEFAULT 'system';

-- バックフィル後はデフォルト値を削除する
ALTER TABLE saga.saga_step_logs
    ALTER COLUMN tenant_id DROP DEFAULT;

-- saga_step_logs の tenant_id インデックスを追加する
CREATE INDEX IF NOT EXISTS idx_saga_step_logs_tenant_id
    ON saga.saga_step_logs (tenant_id);

-- saga_states テーブルの RLS を有効化する
ALTER TABLE saga.saga_states ENABLE ROW LEVEL SECURITY;

-- テナント分離ポリシーを設定する（app.current_tenant_id セッション変数でフィルタリング）
-- current_setting の第 2 引数 true = 変数未設定時に NULL を返すことでエラーを回避する
DROP POLICY IF EXISTS tenant_isolation ON saga.saga_states;
CREATE POLICY tenant_isolation ON saga.saga_states
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);

-- saga_step_logs テーブルの RLS を有効化する
ALTER TABLE saga.saga_step_logs ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON saga.saga_step_logs;
CREATE POLICY tenant_isolation ON saga.saga_step_logs
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);

-- スーパーユーザー・オーナーロールも RLS の適用対象とする（バイパスを防止）
ALTER TABLE saga.saga_states FORCE ROW LEVEL SECURITY;
ALTER TABLE saga.saga_step_logs FORCE ROW LEVEL SECURITY;

COMMIT;
