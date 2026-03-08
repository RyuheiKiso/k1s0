CREATE SCHEMA IF NOT EXISTS order_service;

SET search_path TO order_service;

CREATE TABLE IF NOT EXISTS orders (
    id            UUID         PRIMARY KEY,
    customer_id   TEXT         NOT NULL,
    status        TEXT         NOT NULL DEFAULT 'pending',
    total_amount  BIGINT       NOT NULL DEFAULT 0,
    currency      TEXT         NOT NULL DEFAULT 'JPY',
    notes         TEXT,
    created_by    TEXT         NOT NULL,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- 顧客IDでの検索用インデックス
CREATE INDEX IF NOT EXISTS idx_orders_customer_id ON orders (customer_id);

-- ステータスでのフィルタリング用インデックス
CREATE INDEX IF NOT EXISTS idx_orders_status ON orders (status);

-- 作成日時での並び替え用インデックス
CREATE INDEX IF NOT EXISTS idx_orders_created_at ON orders (created_at DESC);
