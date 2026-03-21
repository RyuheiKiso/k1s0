-- マルチテナント対応ロールバック: RLS ポリシーと tenant_id カラムを削除する。
-- ロールバック時は RLS → インデックス → カラムの順に削除する。

SET search_path TO order_service;

-- order_items の RLS を無効化する
ALTER TABLE order_items DISABLE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS tenant_isolation ON order_items;

-- orders の RLS を無効化する
ALTER TABLE orders DISABLE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS tenant_isolation ON orders;

-- インデックスを削除する
DROP INDEX IF EXISTS idx_orders_tenant_id;
DROP INDEX IF EXISTS idx_orders_tenant_customer;
DROP INDEX IF EXISTS idx_order_items_tenant_id;

-- tenant_id カラムを削除する
ALTER TABLE order_items DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE orders DROP COLUMN IF EXISTS tenant_id;
