SET search_path TO order_service;

ALTER TABLE orders
    DROP COLUMN IF EXISTS updated_by,
    DROP COLUMN IF EXISTS version;
