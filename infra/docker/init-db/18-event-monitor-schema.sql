-- infra/docker/init-db/18-event-monitor-schema.sql
-- スキーマ定義はマイグレーション（regions/system/server/rust/event-monitor/...）が担当する。
-- 本ファイルは DB 接続先の切り替えとスキーマ・拡張機能の初期作成・権限設定のみを行う。
-- CREATE TABLE / ALTER TABLE / CREATE INDEX / CREATE TRIGGER は含まない。
-- event_monitor スキーマは k1s0_system データベース内に作成する

\c k1s0_system;

-- pgcrypto 拡張（gen_random_uuid 関数に必要）
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- event_monitor スキーマの作成（マイグレーション実行前にスキーマが存在する必要がある）
CREATE SCHEMA IF NOT EXISTS event_monitor;

-- k1s0_event_monitor_rw 専用ロールへのスキーマ・テーブル権限付与（C-08 監査対応: 最小権限の原則）
GRANT USAGE ON SCHEMA event_monitor TO k1s0_event_monitor_rw;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA event_monitor TO k1s0_event_monitor_rw;
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA event_monitor TO k1s0_event_monitor_rw;
-- 将来追加されるテーブルにも自動で権限を付与する
ALTER DEFAULT PRIVILEGES IN SCHEMA event_monitor
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO k1s0_event_monitor_rw;
-- マイグレーション用ロールにもスキーマ操作権限を付与する
GRANT ALL ON SCHEMA event_monitor TO k1s0_migration;
-- HIGH-005 監査対応: k1s0 アプリケーションユーザーに k1s0_event_monitor_rw ロールを付与する
-- これがなければ k1s0 ユーザーは event_monitor スキーマのテーブルにアクセスできない
GRANT k1s0_event_monitor_rw TO k1s0;
