#!/bin/bash
# Keycloak起動前にrealm JSONファイルの環境変数プレースホルダーを展開する
# envsubstを使用して${VAR:-default}形式の変数をシェル環境変数の値に置換する

set -euo pipefail

IMPORT_DIR="/opt/keycloak/data/import"

# envsubstがインストールされていない場合はgettext-baseをインストール
if ! command -v envsubst &> /dev/null; then
  echo "[entrypoint-wrapper] envsubst not found, using sed fallback"
  # envsubstが使えない場合、sedでデフォルト値付き変数を展開するフォールバック
  for f in "${IMPORT_DIR}"/*.json; do
    [ -f "$f" ] || continue
    # ${VAR:-default} パターンを処理: 環境変数が設定されていればその値、なければデフォルト値
    tmpfile=$(mktemp)
    cp "$f" "$tmpfile"
    # shellcheck disable=SC2016
    while IFS= read -r varname; do
      default=$(grep -oP "\\\$\{${varname}:-\K[^}]+" "$tmpfile" | head -1)
      value="${!varname:-$default}"
      sed -i "s|\${${varname}:-${default}}|${value}|g" "$tmpfile"
    done < <(grep -oP '\$\{\K[A-Z_]+(?=:-)' "$tmpfile" | sort -u)
    mv "$tmpfile" "$f"
    echo "[entrypoint-wrapper] Processed $f (sed fallback)"
  done
else
  # envsubstで環境変数プレースホルダーを展開
  for f in "${IMPORT_DIR}"/*.json; do
    [ -f "$f" ] || continue
    tmpfile=$(mktemp)
    envsubst < "$f" > "$tmpfile"
    mv "$tmpfile" "$f"
    echo "[entrypoint-wrapper] Processed $f"
  done
fi

# Keycloak本体を起動（元のコマンド引数をそのまま渡す）
exec /opt/keycloak/bin/kc.sh "$@"
