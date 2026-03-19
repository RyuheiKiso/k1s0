CREATE SCHEMA IF NOT EXISTS inventory_service;
SET search_path TO inventory_service;

CREATE TABLE IF NOT EXISTS inventory_items (
    id UUID PRIMARY KEY,
    product_id TEXT NOT NULL,
    warehouse_id TEXT NOT NULL,
    qty_available INT NOT NULL DEFAULT 0,
    qty_reserved INT NOT NULL DEFAULT 0,
    version INT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_product_warehouse UNIQUE (product_id, warehouse_id),
    CONSTRAINT chk_qty_available CHECK (qty_available >= 0),
    CONSTRAINT chk_qty_reserved CHECK (qty_reserved >= 0)
);

CREATE INDEX IF NOT EXISTS idx_inventory_product ON inventory_items (product_id);
CREATE INDEX IF NOT EXISTS idx_inventory_warehouse ON inventory_items (warehouse_id);
