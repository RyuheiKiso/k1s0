-- Order Service (service tier)
\c k1s0_order;

CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE SCHEMA IF NOT EXISTS order_service;

CREATE TABLE order_service.orders (
    id            UUID         PRIMARY KEY,
    customer_id   TEXT         NOT NULL,
    status        TEXT         NOT NULL DEFAULT 'pending',
    total_amount  BIGINT       NOT NULL DEFAULT 0,
    currency      TEXT         NOT NULL DEFAULT 'JPY',
    notes         TEXT,
    created_by    TEXT         NOT NULL,
    updated_by    VARCHAR(255),
    version       INT          NOT NULL DEFAULT 1,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_orders_customer_id ON order_service.orders(customer_id);
CREATE INDEX idx_orders_status ON order_service.orders(status);
CREATE INDEX idx_orders_created_at ON order_service.orders(created_at DESC);

CREATE TABLE order_service.order_items (
    id            UUID         PRIMARY KEY,
    order_id      UUID         NOT NULL REFERENCES order_service.orders(id) ON DELETE CASCADE,
    product_id    TEXT         NOT NULL,
    product_name  TEXT         NOT NULL,
    quantity      INT          NOT NULL CHECK (quantity > 0),
    unit_price    BIGINT       NOT NULL CHECK (unit_price >= 0),
    subtotal      BIGINT       NOT NULL,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_order_items_order_id ON order_service.order_items(order_id);
CREATE INDEX idx_order_items_product_id ON order_service.order_items(product_id);

CREATE TABLE order_service.outbox_events (
    id             UUID         PRIMARY KEY,
    aggregate_type TEXT         NOT NULL,
    aggregate_id   TEXT         NOT NULL,
    event_type     TEXT         NOT NULL,
    payload        JSONB        NOT NULL,
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    published_at   TIMESTAMPTZ
);

CREATE INDEX idx_outbox_unpublished
    ON order_service.outbox_events(created_at)
    WHERE published_at IS NULL;
