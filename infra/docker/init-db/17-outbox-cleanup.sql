-- infra/docker/init-db/17-outbox-cleanup.sql
-- スキーマ定義はマイグレーション（各サービスのマイグレーション）が担当する。
-- 本ファイルは DB 接続先の切り替えのみを行う。
-- CREATE OR REPLACE PROCEDURE / GRANT EXECUTE はマイグレーション後に適用されるため
-- マイグレーション側（regions/service/*/database/*/migrations/）に移管すること。
-- 現時点では init-db では DDL を実行しない（MEDIUM-10 対応）。

\c k1s0_service;

-- NOTICE: Outbox クリーンアッププロシージャの定義はマイグレーションが担当する。
-- 配信済みイベントを定期削除するプロシージャは以下で管理:
--   regions/service/task/database/postgres/migrations/
--   regions/service/board/database/postgres/migrations/
--   regions/service/activity/database/postgres/migrations/
