-- event-store-db: events テーブルの sequence 列から IDENTITY を削除
-- 005_add_events_sequence_identity.up.sql のロールバック用

ALTER TABLE eventstore.events
    ALTER COLUMN sequence DROP IDENTITY IF EXISTS;
