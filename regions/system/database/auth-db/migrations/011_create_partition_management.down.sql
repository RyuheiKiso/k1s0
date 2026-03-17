-- Revert partition management (pg_partman v5 compatible)

-- pg_partman設定の削除
DELETE FROM partman.part_config
WHERE parent_table = 'auth.audit_logs';

-- パーティションテーブルを通常テーブルに戻す
-- 子テーブルのデタッチ（データは保持）
DO $$
DECLARE
    partition_name TEXT;
BEGIN
    FOR partition_name IN
        SELECT inhrelid::regclass::text
        FROM pg_inherits
        WHERE inhparent = 'auth.audit_logs'::regclass
        ORDER BY inhrelid::regclass::text
    LOOP
        -- %I を使用して識別子を安全にクォートする（SQLインジェクション防止 M-15）
        EXECUTE format('ALTER TABLE auth.audit_logs DETACH PARTITION %I', partition_name);
        EXECUTE format('DROP TABLE IF EXISTS %I', partition_name);
    END LOOP;
END $$;
