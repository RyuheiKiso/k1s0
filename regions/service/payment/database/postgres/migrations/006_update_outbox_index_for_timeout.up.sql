-- H-001 対応: Outbox ポーラーのタイムアウト回復クエリをカバーするようにインデックスを更新する。
-- 旧インデックスは `processing_at IS NULL` のみを部分条件としていたため、
-- タイムアウト回復行（processing_at < NOW() - INTERVAL '5 minutes'）がスキャン対象外だった。
-- PostgreSQL の部分インデックス述語は immutable 式のみ使用可能（NOW() は不可）のため、
-- `WHERE published_at IS NULL` に広げて全未発行行をカバーし、processing_at フィルタは
-- ランタイムに適用させる。
DROP INDEX IF EXISTS idx_outbox_events_unprocessed;
CREATE INDEX IF NOT EXISTS idx_outbox_events_unprocessed
    ON outbox_events (created_at ASC)
    WHERE published_at IS NULL;
