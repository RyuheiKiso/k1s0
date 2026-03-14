SET search_path TO payment_service;

DROP INDEX IF EXISTS idx_outbox_unpublished;
DROP TABLE IF EXISTS outbox_events;
