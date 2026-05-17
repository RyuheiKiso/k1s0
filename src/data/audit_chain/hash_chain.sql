-- ============================================================
-- audit_event hash chain 検証 SQL
-- hash chain の改ざん検知・整合性確認のための関数群を定義する
-- PostgreSQL 16 互換
-- ============================================================

-- search_path をアプリ専用スキーマに固定する
SET search_path = k1s0, public;

-- ============================================================
-- hash chain 整合性確認関数: verify_audit_chain
-- 指定テナントの監査ログ全体の hash chain を検証する
-- ============================================================

-- テナント単位での audit_event hash chain を検証する関数
CREATE OR REPLACE FUNCTION k1s0.verify_audit_chain(
    -- 検証対象のテナント識別子
    p_tenant_id     UUID,
    -- 検証開始時刻（NULL の場合は全期間を対象とする）
    p_from          TIMESTAMPTZ DEFAULT NULL,
    -- 検証終了時刻（NULL の場合は現在時刻まで対象とする）
    p_to            TIMESTAMPTZ DEFAULT NULL
) RETURNS TABLE (
    -- 検証対象のイベント識別子
    event_id        UUID,
    -- イベント記録時刻
    event_at        TIMESTAMPTZ,
    -- 検証結果（true = 正常、false = 破断検知）
    is_valid        BOOLEAN,
    -- 期待されるハッシュ値（前のイベントの current_hash）
    expected_hash   TEXT,
    -- 実際に記録されていた previous_hash
    actual_hash     TEXT
) LANGUAGE plpgsql SECURITY DEFINER STABLE AS $$
BEGIN
    -- 時刻フィルターを考慮した hash chain 検証クエリを実行する
    RETURN QUERY
    WITH ordered_events AS (
        -- テナントの監査イベントを時系列順に取得する
        SELECT
            ae.event_id,
            ae.event_at,
            ae.previous_hash,
            ae.current_hash,
            -- 前のイベントの current_hash を取得（LAG ウィンドウ関数を使用）
            LAG(ae.current_hash) OVER (
                PARTITION BY ae.tenant_id
                ORDER BY ae.event_at, ae.event_id
            ) AS prev_current_hash
        FROM k1s0.audit_event ae
        WHERE ae.tenant_id = p_tenant_id
          -- 開始時刻フィルター（NULL の場合は全期間）
          AND (p_from IS NULL OR ae.event_at >= p_from)
          -- 終了時刻フィルター（NULL の場合は現在時刻まで）
          AND (p_to IS NULL OR ae.event_at <= p_to)
    )
    -- 各イベントの hash chain 整合性を判定する
    SELECT
        oe.event_id,
        oe.event_at,
        -- 整合性判定: previous_hash が NULL の場合（チェーン先頭）は常に valid
        -- それ以外は previous_hash と前のイベントの current_hash が一致することを確認
        CASE
            WHEN oe.previous_hash IS NULL AND oe.prev_current_hash IS NULL THEN true
            WHEN oe.previous_hash = oe.prev_current_hash THEN true
            ELSE false
        END AS is_valid,
        -- 期待されるハッシュ値（前のイベントの current_hash）
        oe.prev_current_hash AS expected_hash,
        -- 実際に記録されていた previous_hash
        oe.previous_hash AS actual_hash
    FROM ordered_events oe
    -- 不整合が検出された行を先に並べる（問題箇所をすぐに確認できるようにする）
    ORDER BY oe.event_at, oe.event_id;
END;
$$;

-- k1s0app ロールに hash chain 検証関数の実行権限を付与する
GRANT EXECUTE ON FUNCTION k1s0.verify_audit_chain(UUID, TIMESTAMPTZ, TIMESTAMPTZ) TO k1s0app;

-- ============================================================
-- hash chain サマリー関数: audit_chain_summary
-- 指定テナントの hash chain 状態を集計した結果を返す
-- ============================================================

-- テナントの audit_event hash chain 状態サマリーを返す関数
CREATE OR REPLACE FUNCTION k1s0.audit_chain_summary(
    -- サマリーを取得するテナント識別子
    p_tenant_id     UUID
) RETURNS TABLE (
    -- テナント識別子
    tenant_id       UUID,
    -- 対象イベントの総数
    total_events    BIGINT,
    -- hash chain が正常なイベント数
    valid_count     BIGINT,
    -- hash chain に破断があるイベント数
    broken_count    BIGINT,
    -- 最古のイベント時刻（hash chain の開始点）
    chain_start_at  TIMESTAMPTZ,
    -- 最新のイベント時刻（hash chain の現在の末尾）
    chain_end_at    TIMESTAMPTZ,
    -- hash chain の状態（'intact' = 正常、'broken' = 破断あり）
    chain_status    TEXT
) LANGUAGE plpgsql SECURITY DEFINER STABLE AS $$
BEGIN
    -- hash chain 全体のサマリー集計クエリを実行する
    RETURN QUERY
    WITH chain_validation AS (
        -- verify_audit_chain 関数を呼び出して各イベントの整合性を取得する
        SELECT * FROM k1s0.verify_audit_chain(p_tenant_id)
    )
    SELECT
        -- テナント識別子を返す
        p_tenant_id AS tenant_id,
        -- 対象イベントの総数を集計する
        COUNT(*) AS total_events,
        -- 正常なイベントの数を集計する
        COUNT(*) FILTER (WHERE is_valid = true) AS valid_count,
        -- 破断があるイベントの数を集計する
        COUNT(*) FILTER (WHERE is_valid = false) AS broken_count,
        -- チェーン開始時刻（最古のイベント）を取得する
        MIN(event_at) AS chain_start_at,
        -- チェーン終了時刻（最新のイベント）を取得する
        MAX(event_at) AS chain_end_at,
        -- チェーン状態を判定する（破断がなければ 'intact'、あれば 'broken'）
        CASE
            WHEN COUNT(*) FILTER (WHERE is_valid = false) = 0 THEN 'intact'
            ELSE 'broken'
        END AS chain_status
    FROM chain_validation;
END;
$$;

-- k1s0app ロールに hash chain サマリー関数の実行権限を付与する
GRANT EXECUTE ON FUNCTION k1s0.audit_chain_summary(UUID) TO k1s0app;

-- ============================================================
-- 全テナントのサマリービュー（監視ダッシュボード用）
-- ============================================================

-- 全テナントの hash chain 状態を集計するビュー（Prometheus exporter が参照）
CREATE OR REPLACE VIEW k1s0.all_tenant_chain_status AS
    -- 各テナントの最新監査イベントを取得して hash chain の状態を簡易確認する
    SELECT
        tenant_id,
        -- 監査イベントの総数（テナントごとの監査量を把握する）
        COUNT(*) AS total_events,
        -- 最新イベントの時刻（監査ログの鮮度確認に使用する）
        MAX(event_at) AS latest_event_at,
        -- hash chain の末尾ハッシュ値（外部検証に使用する）
        (SELECT current_hash
         FROM k1s0.audit_event
         WHERE tenant_id = ae.tenant_id
         ORDER BY event_at DESC, event_id DESC
         LIMIT 1) AS chain_tail_hash
    FROM k1s0.audit_event ae
    GROUP BY tenant_id;

-- k1s0app ロールに全テナントチェーン状態ビューの参照権限を付与する
GRANT SELECT ON k1s0.all_tenant_chain_status TO k1s0app;
