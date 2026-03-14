CREATE SCHEMA IF NOT EXISTS payment_service;
SET search_path TO payment_service;

CREATE TABLE IF NOT EXISTS payments (
    id              UUID         PRIMARY KEY,
    order_id        TEXT         NOT NULL,
    customer_id     TEXT         NOT NULL,
    amount          BIGINT       NOT NULL,
    currency        TEXT         NOT NULL DEFAULT 'JPY',
    status          TEXT         NOT NULL DEFAULT 'initiated',
    payment_method  TEXT,
    transaction_id  TEXT,
    error_code      TEXT,
    error_message   TEXT,
    version         INT          NOT NULL DEFAULT 1,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_payments_order ON payments (order_id);
CREATE INDEX idx_payments_customer ON payments (customer_id);
CREATE INDEX idx_payments_status ON payments (status);
CREATE INDEX idx_payments_created_at ON payments (created_at DESC);
