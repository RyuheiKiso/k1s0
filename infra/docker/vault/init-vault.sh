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

VAULT_ADDR="${VAULT_ADDR:-http://localhost:8200}"
VAULT_TOKEN="${VAULT_TOKEN:-dev-token}"

export VAULT_ADDR VAULT_TOKEN

echo "=== Vault 初期化: ${VAULT_ADDR} ==="

# KV v2 シークレットエンジンの有効化（dev モードでは secret/ がデフォルトで有効）
echo "--- KV v2 シークレットエンジンを確認 ---"
vault secrets list 2>/dev/null | grep -q "^secret/" && echo "secret/ は既に有効" || vault secrets enable -path=secret kv-v2

# === Database 共通設定 ===
echo "--- Database シークレットを登録 ---"
vault kv put secret/k1s0/system/database \
  host=postgres \
  port=5432 \
  username=dev \
  password=dev \
  name=k1s0_system

# === Auth Server ===
echo "--- Auth Server シークレットを登録 ---"
vault kv put secret/k1s0/system/auth-server/database \
  url="postgresql://dev:dev@postgres:5432/k1s0_system?options=-c%20search_path%3Dauth,public" \
  password=dev

vault kv put secret/k1s0/system/auth-server/api-key \
  key="dev-auth-api-key"

# === Config Server ===
echo "--- Config Server シークレットを登録 ---"
vault kv put secret/k1s0/system/config-server/database \
  url="postgresql://dev:dev@postgres:5432/k1s0_system?options=-c%20search_path%3Dconfig,public" \
  password=dev

vault kv put secret/k1s0/system/config-server/api-key \
  key="dev-config-api-key"

# === Saga Server ===
echo "--- Saga Server シークレットを登録 ---"
vault kv put secret/k1s0/system/saga-server/database \
  url="postgresql://dev:dev@postgres:5432/k1s0_system?options=-c%20search_path%3Dsaga,public" \
  password=dev

# === DLQ Manager ===
echo "--- DLQ Manager シークレットを登録 ---"
vault kv put secret/k1s0/system/dlq-manager/database \
  url="postgresql://dev:dev@postgres:5432/k1s0_system?options=-c%20search_path%3Ddlq,public" \
  password=dev

# === Kafka 共通設定 ===
echo "--- Kafka シークレットを登録 ---"
vault kv put secret/k1s0/system/kafka/sasl \
  username="" \
  password="" \
  mechanism="PLAINTEXT"

# === Keycloak クライアントシークレット ===
echo "--- Keycloak シークレットを登録 ---"
vault kv put secret/k1s0/system/keycloak \
  client-id="k1s0-api" \
  client-secret="dev-client-secret"

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
