SET search_path TO inventory_service;

DROP INDEX IF EXISTS idx_inventory_warehouse;
DROP INDEX IF EXISTS idx_inventory_product;
DROP TABLE IF EXISTS inventory_items;
DROP SCHEMA IF EXISTS inventory_service;
