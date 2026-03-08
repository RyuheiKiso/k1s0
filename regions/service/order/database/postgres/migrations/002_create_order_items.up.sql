SET search_path TO order_service;

CREATE TABLE IF NOT EXISTS order_items (
    id            UUID         PRIMARY KEY,
    order_id      UUID         NOT NULL REFERENCES orders (id) ON DELETE CASCADE,
    product_id    TEXT         NOT NULL,
    product_name  TEXT         NOT NULL,
    quantity      INT          NOT NULL CHECK (quantity > 0),
    unit_price    BIGINT       NOT NULL CHECK (unit_price >= 0),
    subtotal      BIGINT       NOT NULL,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- 注文IDでの明細取得用インデックス
CREATE INDEX IF NOT EXISTS idx_order_items_order_id ON order_items (order_id);

-- 商品IDでの検索用インデックス
CREATE INDEX IF NOT EXISTS idx_order_items_product_id ON order_items (product_id);
