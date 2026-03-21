-- マルチテナント対応: payments テーブルに tenant_id カラムを追加し、RLS ポリシーを設定する。
-- 設計根拠: docs/architecture/multi-tenancy.md Phase 1 対応。
-- 既存データは tenant_id = 'system' でバックフィルし、NOT NULL 制約を維持する。
-- RLS ポリシーにより app.current_tenant_id セッション変数でテナントを分離する。

SET search_path TO payment_service;

-- payments テーブルに tenant_id カラムを追加する（既存データのバックフィルとして 'system' をデフォルト値とする）
ALTER TABLE payments
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

-- tenant_id のインデックスを追加してクエリ性能を確保する
CREATE INDEX IF NOT EXISTS idx_payments_tenant_id ON payments (tenant_id);
CREATE INDEX IF NOT EXISTS idx_payments_tenant_customer ON payments (tenant_id, customer_id);

-- payments テーブルの RLS を有効化する
ALTER TABLE payments ENABLE ROW LEVEL SECURITY;

-- テナント分離ポリシーを設定する（app.current_tenant_id セッション変数でフィルタリング）
-- current_setting の第 2 引数 true = 変数未設定時に NULL を返すことでエラーを回避する
DROP POLICY IF EXISTS tenant_isolation ON payments;
CREATE POLICY tenant_isolation ON payments
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);
