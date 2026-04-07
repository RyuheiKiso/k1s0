-- ARCH-HIGH-003 対応: Kafka 送信失敗したイベントを退避するデッドレタキューテーブルを作成する。
-- outbox_poller が Kafka 送信失敗時にこのテーブルへ INSERT し、
-- outbox_events の published_at を現在時刻に更新することで再ポーリングを防止する。
SET LOCAL search_path TO board_service, public;
CREATE TABLE outbox_dead_letter (
    id            UUID          PRIMARY KEY DEFAULT gen_random_uuid(),
    original_id   UUID          NOT NULL,
    event_type    TEXT          NOT NULL,
    payload       JSONB         NOT NULL,
    error_message TEXT          NOT NULL,
    created_at    TIMESTAMPTZ   NOT NULL DEFAULT NOW()
);
-- テナント別・時系列での DLQ 検索を効率化するインデックス
CREATE INDEX idx_dlq_event_type_created ON outbox_dead_letter (event_type, created_at);
