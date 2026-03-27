-- infra/docker/init-db/11-outbox-schema.sql
-- スキーマ定義はマイグレーション（共通 outbox ライブラリ）が担当する。
-- 本ファイルは DB 接続先の切り替えとスキーマ・拡張機能の初期作成のみを行う。
-- CREATE TABLE / CREATE INDEX は含まない。

-- ============================================================
-- k1s0_task データベースに outbox スキーマを作成
-- ============================================================
\c k1s0_task;

-- UUID生成のための拡張機能
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- outbox スキーマを作成（サービス固有スキーマと分離）
-- マイグレーション実行前にスキーマが存在する必要がある
CREATE SCHEMA IF NOT EXISTS outbox;
