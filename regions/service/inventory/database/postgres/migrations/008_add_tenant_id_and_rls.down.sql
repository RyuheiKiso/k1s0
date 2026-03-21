-- マルチテナント対応ロールバック: RLS ポリシーと tenant_id カラムを削除する。
-- ロールバック時は RLS → インデックス → カラムの順に削除する。

SET search_path TO inventory_service;

-- inventory_reservations の RLS を無効化する
ALTER TABLE inventory_reservations DISABLE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS tenant_isolation ON inventory_reservations;

-- inventory_items の RLS を無効化する
ALTER TABLE inventory_items DISABLE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS tenant_isolation ON inventory_items;

-- インデックスを削除する
DROP INDEX IF EXISTS idx_inventory_tenant_id;
DROP INDEX IF EXISTS idx_inventory_tenant_product;
DROP INDEX IF EXISTS idx_reservations_tenant_id;

-- tenant_id カラムを削除する
ALTER TABLE inventory_reservations DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE inventory_items DROP COLUMN IF EXISTS tenant_id;
