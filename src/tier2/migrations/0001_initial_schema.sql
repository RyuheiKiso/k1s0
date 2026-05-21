-- k1s0 tier2 migration: 0001_initial_schema
-- migration_id: 0001_initial_schema
-- description: tier2 の基本スキーマを作成する（domain_event / outbox_message / audit_event の 3 テーブル）
-- safety_level: safe
-- table_class: tenant_scoped
-- applied_at: 2026-05-17T00:00:00Z
--
-- 実行方法:
--   sqlx migrate run --database-url $DATABASE_URL --source src/tier2/migrations/

-- k1s0 スキーマを作成する（tier2 全テーブルはこのスキーマに配置する）
CREATE SCHEMA IF NOT EXISTS k1s0;

-- domain_event テーブル: aggregate の状態変更を永続化する
-- P1 invariant: atomic_triple_write の第 1 INSERT ターゲット
CREATE TABLE IF NOT EXISTS k1s0.domain_event (
    -- エントリの主キー（UUID）
    id UUID PRIMARY KEY,
    -- 関連する aggregate の ID（UUID）
    aggregate_id UUID NOT NULL,
    -- テナント ID（RLS FORCE が参照する）
    tenant_id UUID NOT NULL,
    -- イベント種別（StateChange / DomainEventOccurred 等）
    event_kind TEXT NOT NULL,
    -- イベントペイロード（PII は redact 済み、jsonb 型で格納する）
    payload JSONB NOT NULL,
    -- aggregate バージョン（楽観的ロックに使用する）
    version BIGINT NOT NULL,
    -- 書込日時（UTC タイムゾーン付き）
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- domain_event の tenant_id インデックスを作成する（テナントスコープ検索の高速化）
CREATE INDEX IF NOT EXISTS idx_domain_event_tenant_id
    ON k1s0.domain_event(tenant_id);

-- domain_event の aggregate_id インデックスを作成する（aggregate 単位の検索の高速化）
CREATE INDEX IF NOT EXISTS idx_domain_event_aggregate_id
    ON k1s0.domain_event(aggregate_id);

-- outbox_message テーブル: Debezium CDC 経由で Kafka に転送するための中継テーブル
-- P1 invariant: atomic_triple_write の第 2 INSERT ターゲット
-- P2 invariant: この INSERT が失敗した場合は domain_event の INSERT も rollback する
CREATE TABLE IF NOT EXISTS k1s0.outbox_message (
    -- エントリの主キー（UUID）
    id UUID PRIMARY KEY,
    -- 関連する aggregate の ID（UUID）
    aggregate_id UUID NOT NULL,
    -- テナント ID（Debezium CDC が Kafka routing に使用する）
    tenant_id UUID NOT NULL,
    -- イベント種別（Debezium が Kafka topic routing に使用する）
    event_kind TEXT NOT NULL,
    -- Kafka に転送するペイロード（PII は構造的に不在または redact 済み）
    payload JSONB NOT NULL,
    -- 冪等性キー（同一メッセージの二重投入を防止する、24 時間 TTL）
    idempotency_key TEXT NOT NULL,
    -- 書込日時（UTC タイムゾーン付き）
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Debezium が処理済みにする日時（null = 未処理）
    processed_at TIMESTAMPTZ NULL
);

-- outbox_message の tenant_id インデックスを作成する（テナントスコープ検索の高速化）
CREATE INDEX IF NOT EXISTS idx_outbox_message_tenant_id
    ON k1s0.outbox_message(tenant_id);

-- outbox_message の未処理エントリ検索インデックスを作成する（processed_at が null のエントリ）
CREATE INDEX IF NOT EXISTS idx_outbox_message_unprocessed
    ON k1s0.outbox_message(tenant_id) WHERE processed_at IS NULL;

-- outbox_message の冪等性キーの一意制約を作成する（ダブル書込み防止）
CREATE UNIQUE INDEX IF NOT EXISTS idx_outbox_message_idempotency_key
    ON k1s0.outbox_message(idempotency_key);

-- audit_event テーブル: 全操作を監査記録する
-- P1 invariant: atomic_triple_write の第 3 INSERT ターゲット
-- P4 invariant: pii_segregated テーブルへのアクセスは必ず audit_event に記録する
CREATE TABLE IF NOT EXISTS k1s0.audit_event (
    -- エントリの主キー（domain_event.id と同一 UUID を使用する）
    id UUID PRIMARY KEY,
    -- 関連する aggregate の ID（UUID）
    aggregate_id UUID NOT NULL,
    -- テナント ID（RLS FORCE が参照する）
    tenant_id UUID NOT NULL,
    -- アクター識別子（Keycloak subject、監査ログに記録する）
    actor_id TEXT NOT NULL,
    -- セッション目的（business_op / support / export / migration / emergency）
    purpose TEXT NOT NULL,
    -- テーブルクラス（TenantScoped / TenantMaster / PlatformGlobal / PiiSegregated）
    table_class TEXT NOT NULL,
    -- 操作内容のペイロード（PII は redact 済み）
    payload JSONB NOT NULL,
    -- 書込日時（UTC タイムゾーン付き）
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- audit_event の tenant_id インデックスを作成する（テナントスコープ監査検索の高速化）
CREATE INDEX IF NOT EXISTS idx_audit_event_tenant_id
    ON k1s0.audit_event(tenant_id);

-- audit_event の aggregate_id インデックスを作成する（aggregate 単位の監査検索の高速化）
CREATE INDEX IF NOT EXISTS idx_audit_event_aggregate_id
    ON k1s0.audit_event(aggregate_id);

-- audit_event の actor_id インデックスを作成する（アクター別監査検索の高速化）
CREATE INDEX IF NOT EXISTS idx_audit_event_actor_id
    ON k1s0.audit_event(actor_id);
