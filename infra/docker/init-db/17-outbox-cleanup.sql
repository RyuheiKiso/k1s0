-- Outbox テーブルの配信済みイベントを定期削除するストアドプロシージャ
-- 高負荷環境では配信済みイベントが無限増殖し DB パフォーマンスを劣化させるため、
-- cleanup プロシージャで定期的に削除する。（SM-2 監査対応）
-- 呼び出し方法: scheduler サービスまたは pg_cron による定期実行を想定する。

\c k1s0_service;

-- task_service の Outbox クリーンアッププロシージャ
-- 配信済み（published_at IS NOT NULL）かつ 7 日以上経過したイベントを削除する
CREATE OR REPLACE PROCEDURE task_service.cleanup_outbox_events(retention_days INTEGER DEFAULT 7)
LANGUAGE plpgsql AS $$
BEGIN
    DELETE FROM task_service.outbox_events
    WHERE published_at IS NOT NULL
      AND published_at < NOW() - (retention_days || ' days')::INTERVAL;
END;
$$;

-- board_service の Outbox クリーンアッププロシージャ
CREATE OR REPLACE PROCEDURE board_service.cleanup_outbox_events(retention_days INTEGER DEFAULT 7)
LANGUAGE plpgsql AS $$
BEGIN
    DELETE FROM board_service.outbox_events
    WHERE published_at IS NOT NULL
      AND published_at < NOW() - (retention_days || ' days')::INTERVAL;
END;
$$;

-- activity_service の Outbox クリーンアッププロシージャ
CREATE OR REPLACE PROCEDURE activity_service.cleanup_outbox_events(retention_days INTEGER DEFAULT 7)
LANGUAGE plpgsql AS $$
BEGIN
    DELETE FROM activity_service.outbox_events
    WHERE published_at IS NOT NULL
      AND published_at < NOW() - (retention_days || ' days')::INTERVAL;
END;
$$;

-- 各プロシージャへの実行権限を k1s0 ロールに付与する
GRANT EXECUTE ON PROCEDURE task_service.cleanup_outbox_events(INTEGER) TO k1s0;
GRANT EXECUTE ON PROCEDURE board_service.cleanup_outbox_events(INTEGER) TO k1s0;
GRANT EXECUTE ON PROCEDURE activity_service.cleanup_outbox_events(INTEGER) TO k1s0;
