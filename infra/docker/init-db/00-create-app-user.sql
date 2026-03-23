-- infra/docker/init-db/00-create-app-user.sql
-- アプリケーション接続用の非特権ロールを作成する。
-- NOSUPERUSER, NOBYPASSRLS, LOGIN で RLS が正常に機能する最小権限ロール。
-- パスワードは本番環境では環境変数 DB_PASSWORD で上書きすること。
CREATE ROLE k1s0 WITH LOGIN PASSWORD 'changeme_in_production'
  NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;
