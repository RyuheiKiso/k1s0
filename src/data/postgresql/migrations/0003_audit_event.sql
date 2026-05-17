-- ============================================================
-- migration 0003_audit_event.sql: 追記専用 WORM 監査テーブル
-- hash chain で改ざん検知を実現するイミュータブルなイベントログ
-- PostgreSQL 16 互換
-- ============================================================

-- search_path をアプリ専用スキーマに固定
SET search_path = k1s0, public;

-- ============================================================
-- audit_event テーブル: 追記専用 WORM テーブル
-- UPDATE・DELETE は RLS ポリシーで物理禁止する
-- ============================================================

-- audit_event テーブルを作成（監査証跡の不変ストア）
CREATE TABLE k1s0.audit_event (
    -- 監査イベントの一意識別子（UUID v4 で自動生成）
    event_id        UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    -- イベント記録時刻（clock_timestamp で単調増加を保証）
    event_at        TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    -- テナント識別子（RLS 分離のキー）
    tenant_id       UUID        NOT NULL,
    -- 監査イベント種別（例: "row_inserted", "role_granted", "login_success"）
    event_kind      TEXT        NOT NULL,
    -- イベント発生元のサービス名またはユーザー識別子
    actor           TEXT        NOT NULL,
    -- イベント対象のリソース識別子（テーブル名・集約 ID 等）
    resource        TEXT,
    -- 監査イベントの詳細データ（構造化 JSON）
    payload         JSONB       NOT NULL,
    -- 前のイベントのハッシュ値（チェーン接続のための参照）
    previous_hash   TEXT,
    -- 現在のイベントのハッシュ値（GENERATED ALWAYS AS で自動計算）
    -- previous_hash + event_id + event_kind + payload::text の SHA-256
    current_hash    TEXT        GENERATED ALWAYS AS (
        -- SHA-256 を計算して hex 文字列に変換（tamper detection に使用）
        encode(sha256(convert_to(
            -- previous_hash が NULL の場合は空文字列に変換
            coalesce(previous_hash, '') ||
            -- event_id を文字列として連結
            event_id::text ||
            -- event_kind を連結
            event_kind ||
            -- payload を JSON 文字列として連結
            payload::text,
            -- UTF-8 エンコーディングで変換
            'UTF8'
        )), 'hex')
    ) STORED
);

-- テナント・時刻複合インデックス: テナント別の監査ログ時系列検索を高速化
CREATE INDEX idx_audit_event_tenant_at
    ON k1s0.audit_event (tenant_id, event_at DESC);

-- event_kind インデックス: 特定種別の監査イベント検索を高速化
CREATE INDEX idx_audit_event_kind
    ON k1s0.audit_event (event_kind, event_at DESC);

-- actor インデックス: 特定アクターによる操作履歴検索を高速化
CREATE INDEX idx_audit_event_actor
    ON k1s0.audit_event (actor, event_at DESC);

-- ============================================================
-- audit_event テーブルへの RLS 適用（WORM 強制）
-- ============================================================

-- audit_event の RLS を有効化
ALTER TABLE k1s0.audit_event ENABLE ROW LEVEL SECURITY;

-- FORCE RLS: superuser も含む全ロールにポリシーを強制適用
ALTER TABLE k1s0.audit_event FORCE ROW LEVEL SECURITY;

-- 読み取りポリシー: 自テナントの監査ログのみ参照可能
CREATE POLICY audit_event_select_policy ON k1s0.audit_event
    AS RESTRICTIVE
    FOR SELECT
    -- テナント分離: GUC と一致するテナントの監査ログのみ表示
    USING (tenant_id = k1s0.current_tenant_id());

-- 挿入ポリシー: 自テナント向けの監査イベントのみ追記可能
CREATE POLICY audit_event_insert_policy ON k1s0.audit_event
    AS RESTRICTIVE
    FOR INSERT
    -- WITH CHECK で挿入行のテナント ID を検証
    WITH CHECK (tenant_id = k1s0.current_tenant_id());

-- UPDATE を全行で拒否（WORM 保証: 一度記録した監査ログは変更不可）
CREATE POLICY audit_event_deny_update_policy ON k1s0.audit_event
    AS RESTRICTIVE
    FOR UPDATE
    -- USING FALSE で全行の UPDATE を物理禁止
    USING (false);

-- DELETE を全行で拒否（WORM 保証: 一度記録した監査ログは削除不可）
CREATE POLICY audit_event_deny_delete_policy ON k1s0.audit_event
    AS RESTRICTIVE
    FOR DELETE
    -- USING FALSE で全行の DELETE を物理禁止
    USING (false);

-- ============================================================
-- k1s0app ロールへの権限付与
-- ============================================================

-- k1s0app ロールに読み取りと挿入のみ許可（UPDATE・DELETE は RLS が禁止）
GRANT SELECT, INSERT ON TABLE k1s0.audit_event TO k1s0app;

-- ============================================================
-- 監査イベント挿入用のヘルパー関数
-- ============================================================

-- 監査イベントを安全に挿入し、hash chain を自動接続する関数
CREATE OR REPLACE FUNCTION k1s0.insert_audit_event(
    -- 監査イベント種別
    p_event_kind    TEXT,
    -- イベント発生元のアクター識別子
    p_actor         TEXT,
    -- イベント対象のリソース（nullable）
    p_resource      TEXT,
    -- 監査イベントの詳細ペイロード
    p_payload       JSONB
) RETURNS UUID
    LANGUAGE plpgsql
    SECURITY DEFINER
AS $$
DECLARE
    -- 直前の監査イベントのハッシュ値を格納する変数
    v_previous_hash TEXT;
    -- 新規挿入した監査イベントの ID を格納する変数
    v_event_id      UUID;
    -- テナント ID を GUC から取得する変数
    v_tenant_id     UUID;
BEGIN
    -- テナント ID を GUC から取得（NULL の場合はアクセス拒否）
    v_tenant_id := k1s0.current_tenant_id();

    -- テナント ID が NULL の場合はエラーを発生させる
    IF v_tenant_id IS NULL THEN
        RAISE EXCEPTION '監査イベント挿入エラー: app.tenant_id が設定されていません';
    END IF;

    -- 同一テナントの最新監査イベントのハッシュ値を取得（FOR UPDATE で競合を防止）
    SELECT current_hash
    INTO v_previous_hash
    FROM k1s0.audit_event
    WHERE tenant_id = v_tenant_id
    ORDER BY event_at DESC, event_id DESC
    LIMIT 1
    FOR UPDATE SKIP LOCKED;

    -- 新しい監査イベントを挿入し、hash chain を接続する
    INSERT INTO k1s0.audit_event (tenant_id, event_kind, actor, resource, payload, previous_hash)
    VALUES (v_tenant_id, p_event_kind, p_actor, p_resource, p_payload, v_previous_hash)
    -- 挿入した行の event_id を返す
    RETURNING event_id INTO v_event_id;

    -- 挿入した監査イベントの ID を返す
    RETURN v_event_id;
END;
$$;

-- k1s0app ロールに監査イベント挿入関数の実行権限を付与
GRANT EXECUTE ON FUNCTION k1s0.insert_audit_event(TEXT, TEXT, TEXT, JSONB) TO k1s0app;
