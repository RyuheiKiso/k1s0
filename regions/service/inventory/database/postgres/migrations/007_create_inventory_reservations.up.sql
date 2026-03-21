-- 在庫予約の注文別追跡テーブル。Saga 補償トランザクションで order_id から予約を逆引きする。
SET search_path TO inventory_service;

CREATE TABLE IF NOT EXISTS inventory_reservations (
    id UUID PRIMARY KEY,
    order_id TEXT NOT NULL,
    inventory_item_id UUID NOT NULL REFERENCES inventory_items(id),
    product_id TEXT NOT NULL,
    warehouse_id TEXT NOT NULL,
    quantity INT NOT NULL CHECK (quantity > 0),
    status TEXT NOT NULL DEFAULT 'reserved' CHECK (status IN ('reserved', 'released', 'confirmed')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- order_id による逆引き検索（Saga 補償用）。status='reserved' の部分インデックス。
CREATE INDEX idx_reservation_order_id_status
    ON inventory_reservations (order_id)
    WHERE status = 'reserved';

-- 同一 order_id + inventory_item_id の重複予約を防止する冪等性保証。
CREATE UNIQUE INDEX uq_reservation_order_item
    ON inventory_reservations (order_id, inventory_item_id);
