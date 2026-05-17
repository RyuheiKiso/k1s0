-- ============================================================
-- migration 0002_rls_force.sql: RLS FORCE 全テーブル適用
-- cross-tenant データ漏洩を物理防止する行レベルセキュリティ設定
-- PostgreSQL 16 互換
-- ============================================================

-- search_path をアプリ専用スキーマに固定
SET search_path = k1s0, public;

-- ============================================================
-- RLS ヘルパー関数の定義
-- session_context から tenant_id を安全に取得する
-- ============================================================

-- tenant_id を GUC から取得する関数（NULL 安全）
-- security definer で実行することで呼び出し元の権限に依存しない
CREATE OR REPLACE FUNCTION k1s0.current_tenant_id()
    RETURNS UUID
    LANGUAGE plpgsql
    SECURITY DEFINER
    STABLE
AS $$
BEGIN
    -- GUC 'app.tenant_id' が設定されていない場合は NULL を返す（アクセス拒否）
    RETURN current_setting('app.tenant_id', true)::UUID;
EXCEPTION
    -- UUID 変換失敗時も NULL を返してアクセスをブロック
    WHEN invalid_text_representation THEN
        RETURN NULL;
    -- GUC が未設定の場合も NULL を返す
    WHEN undefined_object THEN
        RETURN NULL;
END;
$$;

-- ============================================================
-- domain_event テーブルへの RLS 適用
-- ============================================================

-- domain_event の RLS を有効化（通常ユーザーがポリシーの対象になる）
ALTER TABLE k1s0.domain_event ENABLE ROW LEVEL SECURITY;

-- FORCE RLS: superuser も含む全ロールにポリシーを強制適用
ALTER TABLE k1s0.domain_event FORCE ROW LEVEL SECURITY;

-- テナント分離ポリシー: 読み取りは自テナントのデータのみ許可
CREATE POLICY tenant_isolation_select_policy ON k1s0.domain_event
    AS RESTRICTIVE
    FOR SELECT
    -- current_tenant_id() が返す UUID と行の tenant_id が一致する行のみ表示
    USING (tenant_id = k1s0.current_tenant_id());

-- テナント分離ポリシー: 挿入は自テナント向けデータのみ許可
CREATE POLICY tenant_isolation_insert_policy ON k1s0.domain_event
    AS RESTRICTIVE
    FOR INSERT
    -- WITH CHECK で挿入行の tenant_id が GUC と一致することを検証
    WITH CHECK (tenant_id = k1s0.current_tenant_id());

-- domain_event は追記専用: UPDATE を RLS で物理禁止
CREATE POLICY deny_update_policy ON k1s0.domain_event
    AS RESTRICTIVE
    FOR UPDATE
    -- USING FALSE で UPDATE を全行に対して拒否（追記専用 WORM を保証）
    USING (false);

-- domain_event は追記専用: DELETE を RLS で物理禁止
CREATE POLICY deny_delete_policy ON k1s0.domain_event
    AS RESTRICTIVE
    FOR DELETE
    -- USING FALSE で DELETE を全行に対して拒否
    USING (false);

-- ============================================================
-- state_store テーブルへの RLS 適用
-- ============================================================

-- state_store の RLS を有効化
ALTER TABLE k1s0.state_store ENABLE ROW LEVEL SECURITY;

-- FORCE RLS: superuser も含む全ロールにポリシーを強制適用
ALTER TABLE k1s0.state_store FORCE ROW LEVEL SECURITY;

-- テナント分離ポリシー: 読み取りは自テナントのスナップショットのみ
CREATE POLICY tenant_isolation_select_policy ON k1s0.state_store
    AS RESTRICTIVE
    FOR SELECT
    -- tenant_id が GUC と一致するスナップショットのみアクセス可能
    USING (tenant_id = k1s0.current_tenant_id());

-- テナント分離ポリシー: 挿入は自テナント向けのみ許可
CREATE POLICY tenant_isolation_insert_policy ON k1s0.state_store
    AS RESTRICTIVE
    FOR INSERT
    -- 挿入行の tenant_id が GUC と一致することを検証
    WITH CHECK (tenant_id = k1s0.current_tenant_id());

-- state_store の UPDATE: 自テナントのスナップショットのみ更新可能
CREATE POLICY tenant_isolation_update_policy ON k1s0.state_store
    AS RESTRICTIVE
    FOR UPDATE
    -- 更新対象行と更新後の行の両方で tenant_id を検証
    USING (tenant_id = k1s0.current_tenant_id())
    WITH CHECK (tenant_id = k1s0.current_tenant_id());

-- state_store の DELETE: 自テナントのスナップショットのみ削除可能
CREATE POLICY tenant_isolation_delete_policy ON k1s0.state_store
    AS RESTRICTIVE
    FOR DELETE
    -- 削除対象行の tenant_id が GUC と一致することを確認
    USING (tenant_id = k1s0.current_tenant_id());

-- ============================================================
-- outbox_message テーブルへの RLS 適用
-- ============================================================

-- outbox_message の RLS を有効化
ALTER TABLE k1s0.outbox_message ENABLE ROW LEVEL SECURITY;

-- FORCE RLS: superuser も含む全ロールにポリシーを強制適用
ALTER TABLE k1s0.outbox_message FORCE ROW LEVEL SECURITY;

-- テナント分離ポリシー: 読み取りは自テナントの outbox のみ
CREATE POLICY tenant_isolation_select_policy ON k1s0.outbox_message
    AS RESTRICTIVE
    FOR SELECT
    -- tenant_id が GUC と一致するメッセージのみアクセス可能
    USING (tenant_id = k1s0.current_tenant_id());

-- テナント分離ポリシー: 挿入は自テナント向けのみ許可
CREATE POLICY tenant_isolation_insert_policy ON k1s0.outbox_message
    AS RESTRICTIVE
    FOR INSERT
    -- 挿入行の tenant_id が GUC と一致することを検証
    WITH CHECK (tenant_id = k1s0.current_tenant_id());

-- outbox_message の UPDATE: 自テナントのメッセージのみ更新可能（送信フラグ更新等）
CREATE POLICY tenant_isolation_update_policy ON k1s0.outbox_message
    AS RESTRICTIVE
    FOR UPDATE
    -- 更新対象行と更新後の行の両方で tenant_id を検証
    USING (tenant_id = k1s0.current_tenant_id())
    WITH CHECK (tenant_id = k1s0.current_tenant_id());

-- outbox_message の DELETE: 自テナントの送信済みメッセージのみ削除可能
CREATE POLICY tenant_isolation_delete_policy ON k1s0.outbox_message
    AS RESTRICTIVE
    FOR DELETE
    -- sent = true の行のみ削除可能（未送信の誤削除を防止）
    USING (tenant_id = k1s0.current_tenant_id() AND sent = true);

-- ============================================================
-- アプリケーションロールへの権限付与
-- ============================================================

-- k1s0app ロールに domain_event の読み取り・挿入権限を付与（RLS がさらに絞り込む）
GRANT SELECT, INSERT ON TABLE k1s0.domain_event TO k1s0app;

-- k1s0app ロールに state_store の全 CRUD 権限を付与（RLS がテナント分離を保証）
GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE k1s0.state_store TO k1s0app;

-- k1s0app ロールに outbox_message の全 CRUD 権限を付与（RLS がテナント分離を保証）
GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE k1s0.outbox_message TO k1s0app;

-- k1s0app ロールに current_tenant_id 関数の実行権限を付与
GRANT EXECUTE ON FUNCTION k1s0.current_tenant_id() TO k1s0app;

-- ============================================================
-- RLS 適用後の検証クエリ（CI でのスモークテストとして使用）
-- ============================================================

-- domain_event の RLS 設定を確認（rls_enabled = true かつ force_rls = true）
DO $$
DECLARE
    -- RLS 設定確認用の変数
    v_rls_enabled   BOOLEAN;
    -- FORCE RLS 設定確認用の変数
    v_force_rls     BOOLEAN;
BEGIN
    -- domain_event の RLS 状態を pg_class から取得
    SELECT relrowsecurity, relforcerowsecurity
    INTO v_rls_enabled, v_force_rls
    FROM pg_class
    WHERE relname = 'domain_event'
      AND relnamespace = (SELECT oid FROM pg_namespace WHERE nspname = 'k1s0');

    -- RLS が有効化されていない場合はエラーを発生させる
    IF NOT v_rls_enabled THEN
        RAISE EXCEPTION 'domain_event: RLS が有効化されていません';
    END IF;

    -- FORCE RLS が設定されていない場合はエラーを発生させる
    IF NOT v_force_rls THEN
        RAISE EXCEPTION 'domain_event: FORCE RLS が設定されていません';
    END IF;

    -- 検証成功メッセージを出力
    RAISE NOTICE 'domain_event: RLS FORCE 設定が正常に確認されました';
END;
$$;
