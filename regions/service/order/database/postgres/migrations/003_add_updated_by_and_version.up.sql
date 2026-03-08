SET search_path TO order_service;

ALTER TABLE orders
    ADD COLUMN updated_by VARCHAR(255),
    ADD COLUMN version    INT NOT NULL DEFAULT 1;
