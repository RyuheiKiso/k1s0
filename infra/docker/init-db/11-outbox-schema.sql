-- infra/docker/init-db/13-outbox-schema.sql
-- 共通 Outbox ライブラリ (regions/system/library/rust/outbox) が使用する
-- outbox.outbox_messages テーブルを作成する。
-- postgres_store.rs が参照する outbox スキーマ・テーブルの初期化SQL。

-- ============================================================
-- k1s0_task データベースに outbox スキーマを作成
-- ============================================================
\c k1s0_task;

-- UUID生成のための拡張機能
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- outbox スキーマを作成（サービス固有スキーマと分離）
CREATE SCHEMA IF NOT EXISTS outbox;

-- outbox メッセージテーブル
-- 共通 outbox ライブラリがべき等にメッセージを保存・取得・更新・削除するためのテーブル
CREATE TABLE IF NOT EXISTS outbox.outbox_messages (
    -- メッセージの一意識別子
    id              UUID            PRIMARY KEY,
    -- べき等キー（重複送信防止のための UNIQUE 制約）
    idempotency_key VARCHAR(255)    NOT NULL UNIQUE,
    -- メッセージの送信先トピック名
    topic           TEXT            NOT NULL,
    -- パーティション分散用キー
    partition_key   TEXT            NOT NULL,
    -- メッセージ本体（JSON形式）
    payload         JSONB           NOT NULL,
    -- メッセージの処理状態（PENDING / DELIVERED / FAILED）
    status          TEXT            NOT NULL DEFAULT 'PENDING',
    -- 現在のリトライ回数
    retry_count     INT             NOT NULL DEFAULT 0,
    -- 最大リトライ回数
    max_retries     INT             NOT NULL DEFAULT 3,
    -- 最後のエラーメッセージ（失敗時のみ）
    last_error      TEXT,
    -- メッセージ作成日時
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    -- 次回処理可能日時（リトライ時のバックオフ制御に使用）
    process_after   TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

-- fetch_pending クエリの高速化インデックス
-- status が PENDING/FAILED で process_after が現在時刻以前のレコードを効率的に取得する
CREATE INDEX IF NOT EXISTS idx_outbox_messages_pending
    ON outbox.outbox_messages (status, process_after)
    WHERE status IN ('PENDING', 'FAILED');

-- delete_delivered クエリの高速化インデックス
-- DELIVERED 状態のメッセージを created_at で効率的にクリーンアップする
CREATE INDEX IF NOT EXISTS idx_outbox_messages_delivered_cleanup
    ON outbox.outbox_messages (created_at)
    WHERE status = 'DELIVERED';
