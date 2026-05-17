-- ============================================================
-- migration 0004_outbox.sql: Transactional Outbox パターン強化
-- Debezium CDC による at-least-once 配信を保証する追加設定
-- PostgreSQL 16 互換
-- ============================================================

-- search_path をアプリ専用スキーマに固定
SET search_path = k1s0, public;

-- ============================================================
-- outbox_message テーブルの論理レプリケーション設定
-- Debezium が outbox_message を CDC で取得するためのレプリカ識別子設定
-- ============================================================

-- REPLICA IDENTITY FULL: UPDATE・DELETE 時に変更前の全列を WAL に記録
-- Debezium が変更前後の全データを取得するために必要
ALTER TABLE k1s0.outbox_message REPLICA IDENTITY FULL;

-- ============================================================
-- Outbox ルーター関数: イベント種別に応じて Kafka トピックを決定
-- ============================================================

-- Outbox メッセージを domain_event から自動生成する関数
CREATE OR REPLACE FUNCTION k1s0.enqueue_outbox_message(
    -- 送信元の domain_event の識別子
    p_event_id      UUID,
    -- テナント識別子
    p_tenant_id     UUID,
    -- 集約識別子（Kafka パーティションキーとして使用）
    p_aggregate_id  UUID,
    -- イベント種別（トピックルーティングに使用）
    p_event_kind    TEXT,
    -- Avro シリアライズ済みペイロード（バイナリ）
    p_payload       BYTEA
) RETURNS UUID
    LANGUAGE plpgsql
    SECURITY DEFINER
AS $$
DECLARE
    -- 送信先 Kafka トピック名を格納する変数
    v_topic         TEXT;
    -- 挿入した outbox メッセージの ID を格納する変数
    v_message_id    UUID;
BEGIN
    -- イベント種別に基づいて送信先 Kafka トピックを決定
    v_topic := CASE
        -- 監査関連イベントは audit_events トピックに送信
        WHEN p_event_kind LIKE 'audit.%' THEN 'audit_events'
        -- スキーマ変更イベントは schema_changes トピックに送信
        WHEN p_event_kind LIKE 'schema.%' THEN 'schema_changes'
        -- その他の全イベントは domain_events トピックに送信
        ELSE 'domain_events'
    END;

    -- outbox_message テーブルにメッセージを挿入する
    INSERT INTO k1s0.outbox_message (
        event_id,
        tenant_id,
        topic,
        partition_key,
        payload
    ) VALUES (
        p_event_id,
        p_tenant_id,
        v_topic,
        -- パーティションキーとして集約 ID を使用（順序保証）
        p_aggregate_id::text,
        p_payload
    )
    -- 挿入した行のメッセージ ID を返す
    RETURNING message_id INTO v_message_id;

    -- 挿入したメッセージ ID を返す
    RETURN v_message_id;
END;
$$;

-- k1s0app ロールに outbox キューイング関数の実行権限を付与
GRANT EXECUTE ON FUNCTION k1s0.enqueue_outbox_message(UUID, UUID, UUID, TEXT, BYTEA) TO k1s0app;

-- ============================================================
-- 送信済み outbox メッセージのクリーンアップビュー
-- 30 日以上前の送信済みメッセージを削除するためのビュー（Debezium 後処理用）
-- ============================================================

-- 削除対象の送信済みメッセージを定義するビュー（直接削除は Debezium が実行）
CREATE VIEW k1s0.outbox_cleanup_candidates AS
    -- 送信済みかつ 30 日以上前に送信されたメッセージを選択
    SELECT message_id, event_id, tenant_id, sent_at
    FROM k1s0.outbox_message
    WHERE sent = true
      -- 30 日以上前に送信が完了したメッセージのみ対象
      AND sent_at < clock_timestamp() - INTERVAL '30 days';

-- k1s0app ロールにクリーンアップビューの参照権限を付与
GRANT SELECT ON k1s0.outbox_cleanup_candidates TO k1s0app;

-- ============================================================
-- outbox_message の監視クエリ（Prometheus メトリクス収集用）
-- ============================================================

-- 未送信 outbox メッセージ数を集計するビュー（アラート用）
CREATE VIEW k1s0.outbox_pending_count AS
    -- テナント別・トピック別の未送信メッセージ数を集計
    SELECT
        tenant_id,
        topic,
        -- 未送信メッセージの件数
        COUNT(*) AS pending_count,
        -- 最古の未送信メッセージの作成時刻（遅延検知に使用）
        MIN(created_at) AS oldest_pending_at
    FROM k1s0.outbox_message
    WHERE sent = false
    GROUP BY tenant_id, topic;

-- k1s0app ロールに監視ビューの参照権限を付与
GRANT SELECT ON k1s0.outbox_pending_count TO k1s0app;
