-- updated_at 自動更新トリガー関数を order_service スキーマに作成する（べき等性あり）
CREATE OR REPLACE FUNCTION order_service.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- べき等性ガード: トリガーが既に存在する場合は削除してから再作成する
DROP TRIGGER IF EXISTS trg_orders_updated_at ON order_service.orders;

-- orders テーブルの updated_at を更新時に自動設定するトリガー
CREATE TRIGGER trg_orders_updated_at
    BEFORE UPDATE ON order_service.orders
    FOR EACH ROW
    EXECUTE FUNCTION order_service.update_updated_at();
