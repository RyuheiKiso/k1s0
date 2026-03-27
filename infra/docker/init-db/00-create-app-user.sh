#!/bin/bash
# アプリケーション接続用の非特権ロール k1s0 を作成する。
# NOSUPERUSER, NOBYPASSRLS, LOGIN で RLS が正常に機能する最小権限ロール。
#
# パスワードは K1S0_DB_PASSWORD 環境変数から取得する。（H-1 監査対応）
# - Docker init-db: POSTGRES_USER/POSTGRES_DB が docker-entrypoint.sh により自動設定される
# - CI 環境: POSTGRES_USER, POSTGRES_DB, PGHOST, PGPASSWORD, K1S0_DB_PASSWORD を設定すること
# - 本番環境: Terraform または Secret Manager で K1S0_DB_PASSWORD を注入すること
#
# 【注意】SQL スクリプト (.sql) は psql 経由で実行されるため環境変数を展開できない。
# そのため本スクリプトは .sh 形式で提供し、bash の変数展開を利用する。
# psql は PGHOST / PGPORT / PGPASSWORD 環境変数を自動参照するため、
# CI 側でこれらを設定すれば明示的なフラグ不要で接続先を制御できる。
set -e

psql -v ON_ERROR_STOP=1 \
  --username "${POSTGRES_USER:-postgres}" \
  --dbname "${POSTGRES_DB:-postgres}" <<-EOSQL
    -- :? 演算子により未設定時はスクリプトが即座に異常終了する（C-6 監査対応）
    -- デフォルトパスワードを排除し、環境変数が必ず設定されていることを保証する
    CREATE ROLE k1s0 WITH LOGIN PASSWORD '${K1S0_DB_PASSWORD:?'K1S0_DB_PASSWORD must be set'}'
      NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;
EOSQL
