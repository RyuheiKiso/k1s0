-- ============================================================
-- migration 0001_initial.sql: 初期スキーマ作成
-- forward-only migration（rollback 禁止）
-- PostgreSQL 16 互換
-- ============================================================

-- アプリケーション専用スキーマを作成（public スキーマとの混在を防ぐ）
CREATE SCHEMA IF NOT EXISTS k1s0;

-- search_path をアプリ専用スキーマに固定（public スキーマへの誤アクセスを防止）
SET search_path = k1s0, public;

-- pgcrypto 拡張が存在しない場合はインストール（UUID・暗号化関数に必要）
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- uuid-ossp 拡張が存在しない場合はインストール（UUID 生成関数に必要）
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- pgaudit 拡張が存在しない場合はインストール（監査ログに必要）
CREATE EXTENSION IF NOT EXISTS pgaudit;

-- ============================================================
-- domain_event テーブル: ドメインイベントストア
-- イベントソーシングパターンの中核テーブル
-- ============================================================

-- domain_event テーブルを作成（イベントソーシングの Append-Only ストア）
CREATE TABLE k1s0.domain_event (
    -- イベントの一意識別子（UUID v4 で自動生成）
    event_id        UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    -- イベント発生時刻（タイムゾーン付き、clock_timestamp で単調増加を保証）
    event_at        TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    -- テナント識別子（Row Level Security で使用する分離キー）
    tenant_id       UUID        NOT NULL,
    -- 集約識別子（どの集約に対するイベントかを識別）
    aggregate_id    UUID        NOT NULL,
    -- 集約型（例: "Order", "Payment" などのドメイン集約名）
    aggregate_type  TEXT        NOT NULL,
    -- イベント種別（例: "OrderPlaced", "PaymentConfirmed"）
    event_kind      TEXT        NOT NULL,
    -- イベントバージョン（集約の楽観的ロックに使用）
    event_version   BIGINT      NOT NULL,
    -- イベントペイロード（Avro スキーマに従う JSON 形式）
    payload         JSONB       NOT NULL,
    -- イベントメタデータ（相関 ID・causation ID 等を格納）
    metadata        JSONB       NOT NULL DEFAULT '{}',
    -- スキーマ識別子（Apicurio Registry のスキーマ ID を参照）
    schema_id       TEXT        NOT NULL,
    -- 作成者サービス名（どのサービスがイベントを発行したか）
    created_by      TEXT        NOT NULL
);

-- aggregate_id + event_version の一意制約（同一集約の重複バージョンを防止）
ALTER TABLE k1s0.domain_event
    ADD CONSTRAINT domain_event_aggregate_version_unique
    UNIQUE (aggregate_id, event_version);

-- tenant_id インデックス: テナントごとのイベント検索を高速化
CREATE INDEX idx_domain_event_tenant_id
    ON k1s0.domain_event (tenant_id);

-- aggregate_id インデックス: 集約ごとのイベント履歴取得を高速化
CREATE INDEX idx_domain_event_aggregate_id
    ON k1s0.domain_event (aggregate_id, event_version);

-- event_at インデックス: 時系列範囲検索を高速化
CREATE INDEX idx_domain_event_event_at
    ON k1s0.domain_event (event_at DESC);

-- ============================================================
-- state_store テーブル: 集約スナップショットストア
-- イベントリプレイのコスト削減のためにスナップショットを保持
-- ============================================================

-- state_store テーブルを作成（集約の最新状態スナップショット）
CREATE TABLE k1s0.state_store (
    -- スナップショットの一意識別子
    snapshot_id     UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    -- 対応する集約の識別子
    aggregate_id    UUID        NOT NULL,
    -- 集約型（domain_event.aggregate_type と対応）
    aggregate_type  TEXT        NOT NULL,
    -- テナント識別子（RLS 分離のキー）
    tenant_id       UUID        NOT NULL,
    -- このスナップショットが対応する最新のイベントバージョン
    at_version      BIGINT      NOT NULL,
    -- スナップショット取得時刻
    snapshot_at     TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    -- 集約の状態データ（JSON シリアライズされた集約ルートの状態）
    state_data      JSONB       NOT NULL,
    -- 使用したスキーマ識別子（Apicurio Registry を参照）
    schema_id       TEXT        NOT NULL
);

-- aggregate_id 単一最新スナップショット取得インデックス
CREATE UNIQUE INDEX idx_state_store_aggregate_latest
    ON k1s0.state_store (aggregate_id, at_version DESC);

-- tenant_id インデックス: テナント単位のスナップショット検索を高速化
CREATE INDEX idx_state_store_tenant_id
    ON k1s0.state_store (tenant_id);

-- ============================================================
-- outbox_message テーブル: Transactional Outbox パターン
-- at-least-once 配信保証のための中間テーブル
-- ============================================================

-- outbox_message テーブルを作成（Transactional Outbox パターン）
CREATE TABLE k1s0.outbox_message (
    -- メッセージの一意識別子
    message_id      UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    -- 対応するドメインイベント識別子（domain_event との連携）
    event_id        UUID        NOT NULL,
    -- テナント識別子（RLS 分離のキー）
    tenant_id       UUID        NOT NULL,
    -- 送信先 Kafka トピック名
    topic           TEXT        NOT NULL,
    -- Kafka パーティションキー（通常は aggregate_id を使用）
    partition_key   TEXT        NOT NULL,
    -- Avro シリアライズ済みメッセージペイロード（バイナリ）
    payload         BYTEA       NOT NULL,
    -- メッセージ作成時刻
    created_at      TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    -- 送信済みフラグ（true = Kafka への送信完了）
    sent            BOOLEAN     NOT NULL DEFAULT false,
    -- 送信完了時刻（null = 未送信）
    sent_at         TIMESTAMPTZ,
    -- 送信試行回数（リトライ上限超過の検出に使用）
    retry_count     INT         NOT NULL DEFAULT 0,
    -- 次回リトライ許可時刻（指数バックオフで計算）
    retry_after     TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    -- 最終エラーメッセージ（デバッグ用）
    last_error      TEXT
);

-- 未送信メッセージ取得インデックス（sent_at, retry_after で絞り込み）
CREATE INDEX idx_outbox_message_pending
    ON k1s0.outbox_message (sent, retry_after)
    WHERE sent = false;

-- event_id インデックス: domain_event との JOIN を高速化
CREATE INDEX idx_outbox_message_event_id
    ON k1s0.outbox_message (event_id);

-- tenant_id インデックス: テナント単位の outbox 検索を高速化
CREATE INDEX idx_outbox_message_tenant_id
    ON k1s0.outbox_message (tenant_id);

-- migration バージョン記録テーブル（sqlx-cli との連携用）
CREATE TABLE IF NOT EXISTS k1s0._sqlx_migrations (
    -- migration バージョン番号
    version         BIGINT      PRIMARY KEY,
    -- migration の説明
    description     TEXT        NOT NULL,
    -- migration タイプ（"up" のみ: forward-only）
    installed_on    TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- migration 実行時間（マイクロ秒）
    success         BOOLEAN     NOT NULL,
    -- migration ファイルのチェックサム（改ざん検知）
    checksum        BYTEA       NOT NULL,
    -- migration 実行時間（ナノ秒）
    execution_time  BIGINT      NOT NULL
);

-- migration 完了の記録（このスクリプト自体）
INSERT INTO k1s0._sqlx_migrations
    (version, description, success, checksum, execution_time)
VALUES
    (1, 'initial schema setup', true, '\x00'::bytea, 0)
ON CONFLICT DO NOTHING;
