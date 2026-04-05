-- dlq_messages に tenant_id カラムを追加し、RLS でテナント分離を実現する。
-- CRIT-005 監査対応: マルチテナント SaaS として他テナントのデータ参照を防ぐ。
-- 既存データは tenant_id = 'system' でバックフィルし、その後 DEFAULT を削除して NOT NULL を維持する。

BEGIN;

-- dlq_messages テーブルに tenant_id カラムを追加する（既存データのバックフィルとして 'system' をデフォルト値とする）
ALTER TABLE dlq.dlq_messages
    ADD COLUMN IF NOT EXISTS tenant_id VARCHAR(255) NOT NULL DEFAULT 'system';

-- バックフィル後はデフォルト値を削除し、新規挿入時の明示指定を強制する
ALTER TABLE dlq.dlq_messages
    ALTER COLUMN tenant_id DROP DEFAULT;

-- tenant_id のインデックスを追加してクエリ性能を確保する
CREATE INDEX IF NOT EXISTS idx_dlq_messages_tenant_id
    ON dlq.dlq_messages (tenant_id);

-- テナントとステータスの複合インデックスを追加する（テナント横断クエリの高速化）
CREATE INDEX IF NOT EXISTS idx_dlq_messages_tenant_status
    ON dlq.dlq_messages (tenant_id, status);

-- dlq_messages テーブルの RLS を有効化する
ALTER TABLE dlq.dlq_messages ENABLE ROW LEVEL SECURITY;

-- テナント分離ポリシーを設定する（app.current_tenant_id セッション変数でフィルタリング）
DROP POLICY IF EXISTS tenant_isolation ON dlq.dlq_messages;
CREATE POLICY tenant_isolation ON dlq.dlq_messages
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);

-- スーパーユーザー・オーナーロールも RLS の適用対象とする（バイパスを防止）
ALTER TABLE dlq.dlq_messages FORCE ROW LEVEL SECURITY;

COMMIT;
