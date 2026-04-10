-- vault.access_logs テーブルは RLS FORCE が有効（migration 007）。
-- 監査ログ一覧取得は管理・運用目的で全テナントを横断する必要があるため
-- SECURITY DEFINER 関数を作成して RLS をバイパスする。
-- LOW-12 対応の keyset ページネーションもこの関数内で実装する。
BEGIN;

-- 全テナントの監査ログを keyset ページネーションで取得する。
-- after_id_in が NULL の場合は先頭ページを返す。
-- limit_in 件数 + 1 件を取得することで呼び出し側が次ページ判定できる。
CREATE OR REPLACE FUNCTION vault.list_access_logs_all_tenants(
    after_id_in UUID,
    limit_in    BIGINT
)
RETURNS TABLE(
    id         UUID,
    key_path   TEXT,
    action     TEXT,
    actor_id   TEXT,
    ip_address TEXT,
    success    BOOL,
    error_msg  TEXT,
    created_at TIMESTAMPTZ
)
LANGUAGE plpgsql
SECURITY DEFINER
STABLE
SET search_path = vault, public
AS $$
BEGIN
    IF after_id_in IS NOT NULL THEN
        -- カーソルより古い（降順で後続の）レコードを keyset で取得する
        RETURN QUERY
            SELECT al.id, al.key_path, al.action, al.actor_id,
                   al.ip_address, al.success, al.error_msg, al.created_at
            FROM vault.access_logs al
            WHERE (al.created_at, al.id) < (
                SELECT a.created_at, a.id
                FROM vault.access_logs a
                WHERE a.id = after_id_in
            )
            ORDER BY al.created_at DESC, al.id DESC
            LIMIT limit_in;
    ELSE
        -- 先頭ページ: created_at 降順で最新から取得する
        RETURN QUERY
            SELECT al.id, al.key_path, al.action, al.actor_id,
                   al.ip_address, al.success, al.error_msg, al.created_at
            FROM vault.access_logs al
            ORDER BY al.created_at DESC, al.id DESC
            LIMIT limit_in;
    END IF;
END;
$$;

COMMIT;
