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

# 環境変数から取得（未設定時はローカル開発用デフォルト値を使用）
VAULT_ADDR="${VAULT_ADDR:-http://localhost:8200}"
VAULT_TOKEN="${VAULT_TOKEN:-dev-token}"
DB_HOST="${DB_HOST:-postgres}"
DB_PORT="${DB_PORT:-5432}"
DB_USERNAME="${DB_USERNAME:-dev}"
DB_PASSWORD="${DB_PASSWORD:-dev}"
DB_NAME="${DB_NAME:-k1s0_system}"
AUTH_API_KEY="${AUTH_API_KEY:-dev-auth-api-key}"
CONFIG_API_KEY="${CONFIG_API_KEY:-dev-config-api-key}"
KAFKA_SASL_USERNAME="${KAFKA_SASL_USERNAME:-}"
KAFKA_SASL_PASSWORD="${KAFKA_SASL_PASSWORD:-}"
KAFKA_SASL_MECHANISM="${KAFKA_SASL_MECHANISM:-PLAINTEXT}"
KC_CLIENT_ID="${KC_CLIENT_ID:-k1s0-api}"
KC_CLIENT_SECRET="${KC_CLIENT_SECRET:-dev-client-secret}"

export VAULT_ADDR VAULT_TOKEN

echo "=== Vault 初期化: ${VAULT_ADDR} ==="

# KV v2 シークレットエンジンの有効化（dev モードでは secret/ がデフォルトで有効）
echo "--- KV v2 シークレットエンジンを確認 ---"
vault secrets list 2>/dev/null | grep -q "^secret/" && echo "secret/ は既に有効" || vault secrets enable -path=secret kv-v2

# === Database 共通設定 ===
echo "--- Database シークレットを登録 ---"
vault kv put secret/k1s0/system/database \
  host="${DB_HOST}" \
  port="${DB_PORT}" \
  username="${DB_USERNAME}" \
  password="${DB_PASSWORD}" \
  name="${DB_NAME}"

# === Auth Server ===
echo "--- Auth Server シークレットを登録 ---"
vault kv put secret/k1s0/system/auth-server/database \
  url="postgresql://${DB_USERNAME}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}?options=-c%20search_path%3Dauth,public" \
  password="${DB_PASSWORD}"

vault kv put secret/k1s0/system/auth-server/api-key \
  key="${AUTH_API_KEY}"

# === Config Server ===
echo "--- Config Server シークレットを登録 ---"
vault kv put secret/k1s0/system/config-server/database \
  url="postgresql://${DB_USERNAME}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}?options=-c%20search_path%3Dconfig,public" \
  password="${DB_PASSWORD}"

vault kv put secret/k1s0/system/config-server/api-key \
  key="${CONFIG_API_KEY}"

# === Saga Server ===
echo "--- Saga Server シークレットを登録 ---"
vault kv put secret/k1s0/system/saga-server/database \
  url="postgresql://${DB_USERNAME}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}?options=-c%20search_path%3Dsaga,public" \
  password="${DB_PASSWORD}"

# === DLQ Manager ===
echo "--- DLQ Manager シークレットを登録 ---"
vault kv put secret/k1s0/system/dlq-manager/database \
  url="postgresql://${DB_USERNAME}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}?options=-c%20search_path%3Ddlq,public" \
  password="${DB_PASSWORD}"

# === Kafka 共通設定 ===
echo "--- Kafka シークレットを登録 ---"
vault kv put secret/k1s0/system/kafka/sasl \
  username="${KAFKA_SASL_USERNAME}" \
  password="${KAFKA_SASL_PASSWORD}" \
  mechanism="${KAFKA_SASL_MECHANISM}"

# === Keycloak クライアントシークレット ===
echo "--- Keycloak シークレットを登録 ---"
vault kv put secret/k1s0/system/keycloak \
  client-id="${KC_CLIENT_ID}" \
  client-secret="${KC_CLIENT_SECRET}"

# === ポリシー登録 ===
echo "--- Vault ポリシーを登録 ---"
if [ -f "infra/vault/policies/k1s0-system.hcl" ]; then
  vault policy write k1s0-system infra/vault/policies/k1s0-system.hcl
fi
if [ -f "infra/vault/policies/saga-server.hcl" ]; then
  vault policy write saga-server infra/vault/policies/saga-server.hcl
fi
if [ -f "infra/vault/policies/dlq-manager.hcl" ]; then
  vault policy write dlq-manager infra/vault/policies/dlq-manager.hcl
fi

echo ""
echo "=== 登録済みシークレットを確認 ==="
vault kv list secret/k1s0/system/ 2>/dev/null || echo "(list not available in dev mode)"

echo ""
echo "=== Vault 初期化完了 ==="
