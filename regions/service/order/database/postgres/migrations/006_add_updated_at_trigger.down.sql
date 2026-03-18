-- updated_at トリガーとトリガー関数を削除する
DROP TRIGGER IF EXISTS trg_orders_updated_at ON order_service.orders;
DROP FUNCTION IF EXISTS order_service.update_updated_at();
