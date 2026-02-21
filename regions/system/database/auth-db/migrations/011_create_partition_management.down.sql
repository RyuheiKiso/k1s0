-- auth-db: audit_logs パーティション管理解除 (pg_partman)
-- pg_partman による自動管理を停止し、パーティション設定を元に戻す。
-- 注意: テーブルデータは保持される (p_keep_table = true)

SELECT partman.undo_partition_proc(
    p_parent_table := 'auth.audit_logs',
    p_keep_table   := true
);

-- pg_partman 拡張の削除は、他テーブルが使用中の場合に影響するためコメントアウト
-- DROP EXTENSION IF EXISTS pg_partman;
