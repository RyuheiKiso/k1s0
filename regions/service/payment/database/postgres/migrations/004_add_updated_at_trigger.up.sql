-- updated_at 自動更新トリガー関数を payment_service スキーマに作成する
CREATE OR REPLACE FUNCTION payment_service.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- payments テーブルの updated_at を更新時に自動設定するトリガー
CREATE TRIGGER trg_payments_updated_at
    BEFORE UPDATE ON payment_service.payments
    FOR EACH ROW
    EXECUTE FUNCTION payment_service.update_updated_at();
