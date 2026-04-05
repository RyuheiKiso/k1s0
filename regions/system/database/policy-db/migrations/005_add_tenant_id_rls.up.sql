-- policies と policy_bundles に tenant_id カラムを追加し、RLS でテナント分離を実現する。
-- CRIT-005 監査対応: マルチテナント SaaS として他テナントのデータ参照を防ぐ。
-- 既存データは tenant_id = 'system' でバックフィルし、その後 DEFAULT を削除して NOT NULL を維持する。

BEGIN;

-- policies テーブルに tenant_id カラムを追加する（既存データのバックフィルとして 'system' をデフォルト値とする）
ALTER TABLE policy.policies
    ADD COLUMN IF NOT EXISTS tenant_id VARCHAR(255) NOT NULL DEFAULT 'system';

-- バックフィル後はデフォルト値を削除し、新規挿入時の明示指定を強制する
ALTER TABLE policy.policies
    ALTER COLUMN tenant_id DROP DEFAULT;

-- tenant_id のインデックスを追加してクエリ性能を確保する
CREATE INDEX IF NOT EXISTS idx_policies_tenant_id
    ON policy.policies (tenant_id);

-- テナントと有効状態の複合インデックスを追加する（テナント横断クエリの高速化）
CREATE INDEX IF NOT EXISTS idx_policies_tenant_enabled
    ON policy.policies (tenant_id, enabled);

-- policy_bundles テーブルに tenant_id カラムを追加する
ALTER TABLE policy.policy_bundles
    ADD COLUMN IF NOT EXISTS tenant_id VARCHAR(255) NOT NULL DEFAULT 'system';

ALTER TABLE policy.policy_bundles
    ALTER COLUMN tenant_id DROP DEFAULT;

CREATE INDEX IF NOT EXISTS idx_policy_bundles_tenant_id
    ON policy.policy_bundles (tenant_id);

-- policies テーブルの RLS を有効化する
ALTER TABLE policy.policies ENABLE ROW LEVEL SECURITY;

-- テナント分離ポリシーを設定する（app.current_tenant_id セッション変数でフィルタリング）
DROP POLICY IF EXISTS tenant_isolation ON policy.policies;
CREATE POLICY tenant_isolation ON policy.policies
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);

-- policy_bundles テーブルの RLS を有効化する
ALTER TABLE policy.policy_bundles ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON policy.policy_bundles;
CREATE POLICY tenant_isolation ON policy.policy_bundles
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);

-- スーパーユーザー・オーナーロールも RLS の適用対象とする（バイパスを防止）
ALTER TABLE policy.policies FORCE ROW LEVEL SECURITY;
ALTER TABLE policy.policy_bundles FORCE ROW LEVEL SECURITY;

COMMIT;
