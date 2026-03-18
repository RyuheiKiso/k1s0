-- updated_at トリガーとトリガー関数を削除する
DROP TRIGGER IF EXISTS trg_payments_updated_at ON payment_service.payments;
DROP FUNCTION IF EXISTS payment_service.update_updated_at();
