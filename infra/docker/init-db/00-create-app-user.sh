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

# H-10 監査対応: SQL インジェクション対策
# HEREDOC 内でのシェル変数展開はパスワードに特殊文字が含まれる場合にSQL構文エラーや
# インジェクションのリスクがある。psql の -c オプションとドル引用符（$$...$$）を使用して
# パスワード文字列を安全にエスケープする。
# :? 演算子により K1S0_DB_PASSWORD が未設定の場合はスクリプトが即座に異常終了する（C-6 監査対応）。
K1S0_DB_PASSWORD="${K1S0_DB_PASSWORD:?K1S0_DB_PASSWORD must be set}"

psql -v ON_ERROR_STOP=1 \
  --username "${POSTGRES_USER:-postgres}" \
  --dbname "${POSTGRES_DB:-postgres}" \
  -c "CREATE ROLE k1s0 WITH LOGIN PASSWORD \$\$${K1S0_DB_PASSWORD}\$\$ NOINHERIT NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;"
  # HIGH-004 監査対応: NOINHERIT を明示的に指定することで、k1s0 ロールが他ロールのメンバーになった際の
  # 権限自動継承を防止する。PostgreSQL 12+ では SET ROLE による意図しない権限昇格を防ぐために必須。
