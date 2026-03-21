-- inventory_reservations テーブルをロールバックする。
SET search_path TO inventory_service;
DROP TABLE IF EXISTS inventory_reservations;
