-- dlq-db: 古い DLQ メッセージのアーカイブ管理
-- RESOLVED / DEAD ステータスで 30 日以上経過したメッセージを削除するストアドプロシージャ

-- アーカイブ用テーブル（削除前の記録保持）
CREATE TABLE IF NOT EXISTS dlq.dlq_messages_archive (
    LIKE dlq.dlq_messages INCLUDING ALL
);

-- アーカイブ実行プロシージャ
CREATE OR REPLACE PROCEDURE dlq.archive_old_dlq_messages(
    p_retention_days INT DEFAULT 30,
    p_batch_size INT DEFAULT 1000
)
LANGUAGE plpgsql
AS $$
DECLARE
    v_cutoff TIMESTAMPTZ;
    v_archived_count INT := 0;
    v_batch_count INT;
BEGIN
    v_cutoff := NOW() - (p_retention_days || ' days')::INTERVAL;

    -- バッチ処理でアーカイブ
    LOOP
        -- アーカイブテーブルへ移動
        WITH to_archive AS (
            SELECT id
            FROM dlq.dlq_messages
            WHERE status IN ('RESOLVED', 'DEAD')
              AND updated_at < v_cutoff
            LIMIT p_batch_size
            FOR UPDATE SKIP LOCKED
        ),
        archived AS (
            INSERT INTO dlq.dlq_messages_archive
            SELECT m.*
            FROM dlq.dlq_messages m
            INNER JOIN to_archive ta ON m.id = ta.id
            RETURNING 1
        )
        SELECT COUNT(*) INTO v_batch_count FROM archived;

        -- 元テーブルから削除
        DELETE FROM dlq.dlq_messages
        WHERE id IN (
            SELECT id
            FROM dlq.dlq_messages
            WHERE status IN ('RESOLVED', 'DEAD')
              AND updated_at < v_cutoff
            LIMIT p_batch_size
        );

        v_archived_count := v_archived_count + v_batch_count;

        -- バッチが空なら終了
        EXIT WHEN v_batch_count = 0;

        -- 中間コミット
        COMMIT;
    END LOOP;

    RAISE NOTICE 'Archived % DLQ messages older than % days', v_archived_count, p_retention_days;
END;
$$;
