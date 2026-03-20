-- auth-db: スキーマ・拡張機能・共通関数の削除
-- CASCADE を指定して依存オブジェクト（テーブル・関数等）を安全に削除する
DROP FUNCTION IF EXISTS auth.update_updated_at() CASCADE;
DROP SCHEMA IF EXISTS auth CASCADE;
DROP EXTENSION IF EXISTS "pgcrypto" CASCADE;
