-- k1s0 tier2 migration: 0007_audit_local_view_or_table
-- migration_id: 0007_audit_local_view_or_table
-- description: audit_local view を k1s0.audit_event から導出する（PII 不在保証 view / relay.rs 参照）
-- safety_level: safe
-- table_class: tenant_scoped
-- applied_at: 2026-05-17T00:00:00Z
--
-- 実行方法:
--   sqlx migrate run --database-url $DATABASE_URL --source src/tier2/migrations/
--
-- 前提条件:
--   0001_initial_schema.sql で k1s0.audit_event が存在すること
--   0003_audit_hash_chain.sql で hash_digest / prev_digest / chain_sequence が追加済みであること
--
-- relay.rs が参照するカラム（relay.rs PendingAuditEvent 構造体の全フィールド）:
--   id, tenant_id, actor_id, purpose, table_class, created_at,
--   prev_digest, chain_sequence, relayed
-- relay.rs の fetch_pending_events SQL コメント（行 128-133）で確認済み:
--   WHERE relayed = false ORDER BY chain_sequence ASC NULLS LAST LIMIT $1
-- relayed カラムは audit_event 本体には存在しないため view 内で定数 false を初期値として持つ
-- ただし relay.rs の mark_as_relayed は UPDATE audit_local SET relayed = true を実行するため
-- view ではなく relayed カラムを持つ audit_event の ALTER TABLE が必要

-- ============================================================
-- audit_event テーブルに relayed カラムを追加する
-- ============================================================
-- relay.rs の mark_as_relayed が UPDATE audit_local SET relayed = true WHERE id = ANY($1) を実行する
-- view は UPDATE 対象にできないため、relayed は audit_event 本体に持たせる
-- relay.rs が audit_local = view として参照する場合も、updatable view / rule で対応可能だが
-- 実装の単純性を優先し、audit_event 本体への ADD COLUMN で対応する
ALTER TABLE k1s0.audit_event
    -- ClickHouse 転送済みフラグ（false = 未送信、true = 送信済み）
    ADD COLUMN IF NOT EXISTS relayed BOOLEAN NOT NULL DEFAULT FALSE;

-- relayed カラムのインデックスを作成する（未送信イベントの高速検索に使用する）
-- relay.rs の fetch_pending_events が WHERE relayed = false で絞り込むために必要
CREATE INDEX IF NOT EXISTS idx_audit_event_relayed_pending
    ON k1s0.audit_event(tenant_id, chain_sequence)
    -- 未送信イベントのみを対象とする部分インデックス（インデックスサイズを最小化する）
    WHERE relayed = FALSE;

-- relayed 済みイベントのインデックスを作成する（送信済みイベントの検索に使用する）
CREATE INDEX IF NOT EXISTS idx_audit_event_relayed_at
    ON k1s0.audit_event(relayed);

-- ============================================================
-- audit_local view の作成: PII フィールドを除いた audit_event の安全な射影
-- ============================================================
-- relay.rs が参照する view（relay.rs コメント行 130: FROM audit_local）
-- payload カラム（PII 含有可能性あり）を除外して PII 不在を保証する
-- CREATE OR REPLACE VIEW は idempotent（既存 view を上書きする）
CREATE OR REPLACE VIEW k1s0.audit_local AS
SELECT
    -- イベント識別子（domain_event と同一 UUID、relay.rs PendingAuditEvent.id に対応）
    id,
    -- 集約識別子（PII ではない、docs §117 で payload のみ除外と定義されているため含める）
    aggregate_id,
    -- テナント識別子（PII ではない、relay.rs PendingAuditEvent.tenant_id に対応）
    tenant_id,
    -- 操作主体（Keycloak subject、PII ではない、relay.rs PendingAuditEvent.actor_id に対応）
    actor_id,
    -- セッション目的（business_op / support 等、relay.rs PendingAuditEvent.purpose に対応）
    purpose,
    -- テーブルクラス（TenantScoped / PiiSegregated 等、relay.rs PendingAuditEvent.table_class に対応）
    table_class,
    -- ハッシュチェーン値（改ざん検知用、0003 で追加、relay.rs PendingAuditEvent の hash chain に対応）
    hash_digest,
    -- 前エントリのハッシュ値（hash chain の連続性保証、relay.rs PendingAuditEvent.prev_digest に対応）
    prev_digest,
    -- チェーン連番（テナント内での順序保証、relay.rs PendingAuditEvent.chain_sequence に対応）
    chain_sequence,
    -- ClickHouse 転送済みフラグ（relay.rs fetch_pending_events の WHERE relayed = false に対応）
    relayed,
    -- 作成日時（UTC タイムゾーン付き、relay.rs PendingAuditEvent.created_at に対応）
    created_at
FROM k1s0.audit_event;
-- payload カラム（PII 含有可能性あり）は意図的に除外する

-- ============================================================
-- audit_local view へのアクセス権付与
-- ============================================================
-- k1s0app ロールが audit_local view を参照・更新できるよう権限を付与する
DO $$
BEGIN
    -- k1s0app ロールが存在する場合のみ GRANT を実行する
    IF EXISTS (SELECT FROM pg_roles WHERE rolname = 'k1s0app') THEN
        -- audit_local view への SELECT 権限を付与する（fetch_pending_events に必要）
        EXECUTE 'GRANT SELECT ON k1s0.audit_local TO k1s0app';
    END IF;
END;
$$;

-- ============================================================
-- 動作確認用コメント
-- ============================================================
-- relay.rs の fetch_pending_events が実行する SELECT の等価クエリ:
--   SELECT id, tenant_id, actor_id, purpose, table_class, created_at,
--          prev_digest, chain_sequence, relayed
--   FROM k1s0.audit_local
--   WHERE relayed = false
--   ORDER BY chain_sequence ASC NULLS LAST
--   LIMIT $1;
--
-- relay.rs の mark_as_relayed が実行する UPDATE の等価クエリ:
--   UPDATE k1s0.audit_event SET relayed = true WHERE id = ANY($1);
--   （audit_event 本体を直接 UPDATE する。audit_local view 経由では行わない）
