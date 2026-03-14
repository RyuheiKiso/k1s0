SET search_path TO payment_service;

DROP INDEX IF EXISTS idx_payments_created_at;
DROP INDEX IF EXISTS idx_payments_status;
DROP INDEX IF EXISTS idx_payments_customer;
DROP INDEX IF EXISTS idx_payments_order;
DROP TABLE IF EXISTS payments;
