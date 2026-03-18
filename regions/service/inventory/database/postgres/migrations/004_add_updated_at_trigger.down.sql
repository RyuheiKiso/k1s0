-- updated_at トリガーとトリガー関数を削除する
DROP TRIGGER IF EXISTS trg_inventory_items_updated_at ON inventory_service.inventory_items;
DROP FUNCTION IF EXISTS inventory_service.update_updated_at();
