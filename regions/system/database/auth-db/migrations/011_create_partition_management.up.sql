-- auth-db: audit_logs パーティション自動管理 (pg_partman)
-- pg_partman 拡張をインストールし、auth.audit_logs の月次パーティション自動管理を設定する。
-- pg_partman が利用できない環境（テストコンテナ等）では自動的にスキップする。

DO $$
BEGIN
    -- pg_partman 拡張の存在確認
    IF EXISTS (
        SELECT 1 FROM pg_available_extensions WHERE name = 'pg_partman'
    ) THEN
        -- 拡張のインストール
        CREATE EXTENSION IF NOT EXISTS pg_partman SCHEMA partman;

        -- audit_logs テーブルをパーティション自動管理の対象として登録する
        PERFORM partman.create_parent(
            p_parent_table   := 'auth.audit_logs',
            p_control        := 'created_at',
            p_type           := 'native',
            p_interval       := '1 month',
            p_premake        := 3
        );

        -- 保持ポリシーを設定する（24ヶ月超のパーティションを自動デタッチ）
        UPDATE partman.part_config
        SET
            retention                = '24 months',
            retention_keep_table     = false,
            retention_keep_index     = false,
            automatic_maintenance    = 'on',
            infinite_time_partitions = true
        WHERE parent_table = 'auth.audit_logs';

        -- メンテナンスを即時実行して 3ヶ月分のパーティションを事前作成する
        PERFORM partman.run_maintenance(p_parent_table := 'auth.audit_logs');
    ELSE
        RAISE NOTICE 'pg_partman is not available; skipping partition management setup.';
    END IF;
END $$;
