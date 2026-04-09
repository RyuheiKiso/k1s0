#!/bin/bash
# init-vault.sh
# Vault dev モードの初期化スクリプト。
# ローカル開発環境 (docker-compose) で使用する。
#
# 前提: Vault が dev モードで起動済みであること。
#   docker compose --profile infra up -d
#
# 使い方:
#   bash infra/docker/vault/init-vault.sh

set -euo pipefail

# ENVIRONMENT が未設定の場合は development をデフォルトにする。
# これにより set -u で未設定変数エラーを防ぎつつ、環境を明示させる。
ENVIRONMENT="${ENVIRONMENT:-development}"

# development 以外の環境ではすべての必須変数が設定されていることを検証する。
# production/staging に加え、未知の環境値でも安全側に倒す。
if [ "$ENVIRONMENT" != "development" ] && [ "$ENVIRONMENT" != "dev" ] && [ "$ENVIRONMENT" != "local" ]; then
    : "${VAULT_ADDR:?VAULT_ADDR must be set in non-development environments}"
    : "${VAULT_TOKEN:?VAULT_TOKEN must be set in non-development environments}"
    : "${VAULT_INIT_KEY:?VAULT_INIT_KEY must be set in non-development environments}"
    # 本番環境ではシークレット値が明示的に設定されていることを必須とする（C-05 監査対応）
    : "${DB_PASSWORD:?本番環境では DB_PASSWORD を必ず設定してください}"
    : "${AUTH_API_KEY:?本番環境では AUTH_API_KEY を必ず設定してください}"
    : "${CONFIG_API_KEY:?本番環境では CONFIG_API_KEY を必ず設定してください}"
    : "${KC_CLIENT_SECRET:?本番環境では KC_CLIENT_SECRET を必ず設定してください}"
fi

# 環境変数から取得（未設定時はローカル開発用デフォルト値を使用）
VAULT_ADDR="${VAULT_ADDR:-http://localhost:8200}"
VAULT_TOKEN="${VAULT_TOKEN:-dev-token}"
DB_HOST="${DB_HOST:-postgres}"
DB_PORT="${DB_PORT:-5432}"
DB_USERNAME="${DB_USERNAME:-dev}"
DB_PASSWORD="${DB_PASSWORD:-dev}"
DB_NAME="${DB_NAME:-k1s0_system}"
# CRITICAL-003 対応: サービス別 DB 名（infra/docker/init-db/01-create-databases.sql と一致させる）
AUTH_DB_NAME="${AUTH_DB_NAME:-auth_db}"
CONFIG_DB_NAME="${CONFIG_DB_NAME:-config_db}"
SAGA_DB_NAME="${SAGA_DB_NAME:-k1s0_saga}"
DLQ_DB_NAME="${DLQ_DB_NAME:-dlq_db}"
AUTH_API_KEY="${AUTH_API_KEY:-dev-auth-api-key}"
CONFIG_API_KEY="${CONFIG_API_KEY:-dev-config-api-key}"
KAFKA_SASL_USERNAME="${KAFKA_SASL_USERNAME:-}"
KAFKA_SASL_PASSWORD="${KAFKA_SASL_PASSWORD:-}"
KAFKA_SASL_MECHANISM="${KAFKA_SASL_MECHANISM:-PLAINTEXT}"
KC_CLIENT_ID="${KC_CLIENT_ID:-k1s0-api}"
KC_CLIENT_SECRET="${KC_CLIENT_SECRET:-dev-client-secret}"

export VAULT_ADDR VAULT_TOKEN

echo "=== Vault 初期化: ${VAULT_ADDR} ==="

# MED-023 監査対応: ホストの vault CLI への依存を排除し docker exec 経由で実行する
# vault CLI がホストにインストールされていない環境でもスクリプトが動作するようにする。
# VAULT_CONTAINER: docker-compose で起動した vault コンテナ名（デフォルト: k1s0-vault-1）
VAULT_CONTAINER="${VAULT_CONTAINER:-k1s0-vault-1}"

# vault() 関数定義前に外部コマンドとしての vault の存在を確認する。
# 関数定義後に command -v vault を呼ぶとシェル関数自体を検出してしまい
# command vault ... が「外部コマンドなし」でエラーになるため、事前にフラグで確保する。
_VAULT_HOST_CMD="$(command -v vault 2>/dev/null || true)"

# vault() ラッパー: ホストに vault CLI がある場合はそのまま使用し、ない場合は docker exec で実行する
vault() {
  if [ -n "${_VAULT_HOST_CMD}" ]; then
    "${_VAULT_HOST_CMD}" "$@"
  else
    docker exec \
      -e VAULT_ADDR="${VAULT_ADDR}" \
      -e VAULT_TOKEN="${VAULT_TOKEN}" \
      "${VAULT_CONTAINER}" vault "$@"
  fi
}

# KV v2 シークレットエンジンの有効化（dev モードでは secret/ がデフォルトで有効）
echo "--- KV v2 シークレットエンジンを確認 ---"
vault secrets list 2>/dev/null | grep -q "^secret/" && echo "secret/ は既に有効" || vault secrets enable -path=secret kv-v2

# シークレット渡しをファイル経由に変更（コマンドラインに値が現れないように）（C-05 監査対応）
# vault_kv_put: JSON 一時ファイル経由で vault kv put を実行するヘルパー関数
# 引数: $1=Vault パス, $2=JSON 文字列
vault_kv_put() {
  local path="$1"
  local json="$2"
  # ホスト vault CLI の有無は _VAULT_HOST_CMD フラグで判定する（関数定義後の command -v vault はシェル関数を検出するため使用しない）
  # MED-023: docker exec 経由の場合はコンテナ内の一時ファイルに直接書き込んで実行する
  if [ -z "${_VAULT_HOST_CMD}" ]; then
    local container_tmpfile="/tmp/vault-secret-$$.json"
    # コンテナ内に JSON を書き込み、vault kv put で読み込み後に削除する
    docker exec \
      -e VAULT_ADDR="${VAULT_ADDR}" \
      -e VAULT_TOKEN="${VAULT_TOKEN}" \
      "${VAULT_CONTAINER}" sh -c "printf '%s' '${json}' > '${container_tmpfile}' && vault kv put '${path}' @'${container_tmpfile}'; rm -f '${container_tmpfile}'"
  else
    local tmpfile
    tmpfile="$(mktemp)"
    echo "${json}" > "${tmpfile}"
    "${_VAULT_HOST_CMD}" kv put "${path}" @"${tmpfile}"
    shred -u "${tmpfile}" 2>/dev/null || rm -f "${tmpfile}"
  fi
}

# === Database 共通設定 ===
echo "--- Database シークレットを登録 ---"
vault_kv_put secret/k1s0/system/database \
  "{\"host\":\"${DB_HOST}\",\"port\":\"${DB_PORT}\",\"username\":\"${DB_USERNAME}\",\"password\":\"${DB_PASSWORD}\",\"name\":\"${DB_NAME}\"}"

# === Auth Server ===
# CRITICAL-003 対応: auth-server は auth_db に接続する（DB_NAME ではなく AUTH_DB_NAME を使用）
echo "--- Auth Server シークレットを登録 ---"
vault_kv_put secret/k1s0/system/auth-server/database \
  "{\"url\":\"postgresql://${DB_USERNAME}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${AUTH_DB_NAME}?options=-c%20search_path%3Dauth,public\",\"password\":\"${DB_PASSWORD}\"}"

vault_kv_put secret/k1s0/system/auth-server/api-key \
  "{\"key\":\"${AUTH_API_KEY}\"}"

# === Config Server ===
# CRITICAL-003 対応: config-server は config_db に接続する（DB_NAME ではなく CONFIG_DB_NAME を使用）
echo "--- Config Server シークレットを登録 ---"
vault_kv_put secret/k1s0/system/config-server/database \
  "{\"url\":\"postgresql://${DB_USERNAME}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${CONFIG_DB_NAME}?options=-c%20search_path%3Dconfig,public\",\"password\":\"${DB_PASSWORD}\"}"

vault_kv_put secret/k1s0/system/config-server/api-key \
  "{\"key\":\"${CONFIG_API_KEY}\"}"

# === Saga Server ===
# CRITICAL-003 対応: saga-server は k1s0_saga に接続する（DB_NAME ではなく SAGA_DB_NAME を使用）
echo "--- Saga Server シークレットを登録 ---"
vault_kv_put secret/k1s0/system/saga-server/database \
  "{\"url\":\"postgresql://${DB_USERNAME}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${SAGA_DB_NAME}?options=-c%20search_path%3Dsaga,public\",\"password\":\"${DB_PASSWORD}\"}"

# === DLQ Manager ===
# CRITICAL-003 対応: dlq-manager は dlq_db に接続する（DB_NAME ではなく DLQ_DB_NAME を使用）
echo "--- DLQ Manager シークレットを登録 ---"
vault_kv_put secret/k1s0/system/dlq-manager/database \
  "{\"url\":\"postgresql://${DB_USERNAME}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DLQ_DB_NAME}?options=-c%20search_path%3Ddlq,public\",\"password\":\"${DB_PASSWORD}\"}"

# === Kafka 共通設定 ===
echo "--- Kafka シークレットを登録 ---"
vault_kv_put secret/k1s0/system/kafka/sasl \
  "{\"username\":\"${KAFKA_SASL_USERNAME}\",\"password\":\"${KAFKA_SASL_PASSWORD}\",\"mechanism\":\"${KAFKA_SASL_MECHANISM}\"}"

# === Keycloak クライアントシークレット ===
echo "--- Keycloak シークレットを登録 ---"
vault_kv_put secret/k1s0/system/keycloak \
  "{\"client-id\":\"${KC_CLIENT_ID}\",\"client-secret\":\"${KC_CLIENT_SECRET}\"}"

# === ポリシー登録 ===
echo "--- Vault ポリシーを登録 ---"
# vault policy write はファイルパスを引数に取るが、docker exec 経由では
# ホスト側のファイルをコンテナが直接読めないため、vault_policy_write ヘルパーを使用する
vault_policy_write() {
  local name="$1"
  local filepath="$2"
  if [ ! -f "${filepath}" ]; then
    return 0
  fi
  if [ -z "${_VAULT_HOST_CMD}" ]; then
    # docker exec 経由: read-only コンテナには docker cp できないため stdin 経由で渡す
    # vault policy write - で標準入力からポリシーを受け取る
    docker exec -i \
      -e VAULT_ADDR="${VAULT_ADDR}" \
      -e VAULT_TOKEN="${VAULT_TOKEN}" \
      "${VAULT_CONTAINER}" vault policy write "${name}" - < "${filepath}"
  else
    "${_VAULT_HOST_CMD}" policy write "${name}" "${filepath}"
  fi
}
vault_policy_write k1s0-system infra/vault/policies/k1s0-system.hcl
vault_policy_write saga-server infra/vault/policies/saga-server.hcl
vault_policy_write dlq-manager infra/vault/policies/dlq-manager.hcl

echo ""
echo "=== 登録済みシークレットを確認 ==="
vault kv list secret/k1s0/system/ 2>/dev/null || echo "(list not available in dev mode)"

echo ""
echo "=== Vault 初期化完了 ==="
