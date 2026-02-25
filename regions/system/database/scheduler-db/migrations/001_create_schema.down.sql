-- scheduler-db: スキーマ・拡張機能・共通関数の削除

DROP FUNCTION IF EXISTS scheduler.update_updated_at() CASCADE;
DROP SCHEMA IF EXISTS scheduler CASCADE;
