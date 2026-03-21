-- マルチテナント対応ロールバック: RLS ポリシーと tenant_id カラムを削除する。
-- ロールバック時は RLS → インデックス → カラムの順に削除する。

SET search_path TO payment_service;

-- payments の RLS を無効化する
ALTER TABLE payments DISABLE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS tenant_isolation ON payments;

-- インデックスを削除する
DROP INDEX IF EXISTS idx_payments_tenant_id;
DROP INDEX IF EXISTS idx_payments_tenant_customer;

-- tenant_id カラムを削除する
ALTER TABLE payments DROP COLUMN IF EXISTS tenant_id;
