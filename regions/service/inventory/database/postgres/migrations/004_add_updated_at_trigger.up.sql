-- updated_at 自動更新トリガー関数を inventory_service スキーマに作成する
CREATE OR REPLACE FUNCTION inventory_service.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- inventory_items テーブルの updated_at を更新時に自動設定するトリガー
CREATE TRIGGER trg_inventory_items_updated_at
    BEFORE UPDATE ON inventory_service.inventory_items
    FOR EACH ROW
    EXECUTE FUNCTION inventory_service.update_updated_at();
