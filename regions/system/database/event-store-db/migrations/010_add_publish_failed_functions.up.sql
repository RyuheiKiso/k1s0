-- publish_failed イベントを全テナント横断で操作するための SECURITY DEFINER 関数を追加する。
-- 背景: eventstore.events は FORCE ROW LEVEL SECURITY が適用されており、
-- アプリロールは app.current_tenant_id を設定しなければ自テナントの行しか参照できない。
-- 全テナント横断の監視・再送ジョブには RLS バイパスが必要であり、
-- SECURITY DEFINER（関数オーナー = migration 実行者 = スーパーユーザー）パターンを使用する。
-- スーパーユーザーは FORCE ROW LEVEL SECURITY の制約を受けないため、全テナントを参照できる。
-- ADR-0118 Phase A/B 実装の基盤となる。

BEGIN;

-- 全テナント横断で publish_failed 件数をカウントする関数（監視用）
-- アプリロールが呼び出しても、スーパーユーザー権限で RLS をバイパスして集計できる
CREATE OR REPLACE FUNCTION eventstore.count_publish_failed_all_tenants()
RETURNS bigint
LANGUAGE sql
SECURITY DEFINER
STABLE
AS $$
    SELECT COUNT(*)::bigint
    FROM eventstore.events
    WHERE publish_status = 'publish_failed';
$$;

-- 全テナント横断で publish_failed イベントを取得する関数（再送ジョブ用）
-- p_batch_limit: 一度に取得する最大件数（デフォルト: 100、負荷制御のため上限設定）
-- 列名は sqlx::FromRow マッピングで使用するため r_ プレフィックスを付与する
CREATE OR REPLACE FUNCTION eventstore.list_publish_failed_events(
    p_batch_limit integer DEFAULT 100
)
RETURNS TABLE (
    r_stream_id    text,
    r_tenant_id    text,
    r_sequence     bigint,
    r_event_type   text,
    r_version      bigint,
    r_payload      jsonb,
    r_metadata     jsonb,
    r_occurred_at  timestamptz,
    r_stored_at    timestamptz
)
LANGUAGE sql
SECURITY DEFINER
STABLE
AS $$
    SELECT
        stream_id::text,
        tenant_id,
        sequence,
        event_type,
        version,
        payload,
        metadata,
        occurred_at,
        stored_at
    FROM eventstore.events
    WHERE publish_status = 'publish_failed'
    ORDER BY occurred_at
    LIMIT p_batch_limit;
$$;

-- アプリロールに実行権限を付与する（PUBLIC = 全ロール）
-- 関数はパラメータ化されており、取得できる情報は publish_failed の集計・一覧のみ
GRANT EXECUTE ON FUNCTION eventstore.count_publish_failed_all_tenants() TO PUBLIC;
GRANT EXECUTE ON FUNCTION eventstore.list_publish_failed_events(integer) TO PUBLIC;

COMMIT;
