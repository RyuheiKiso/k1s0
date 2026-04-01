#!/usr/bin/env bash
# start-local.sh — k1s0 ローカル開発環境の一括起動
# MEDIUM-001 監査対応: --env-file/.env.dev を全コマンドに追加し、ポート一覧・admin 情報を最新化
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

PROFILES="${1:-all}"

# Docker Compose 共通オプション: 環境変数ファイルと compose ファイルを明示する
DC="docker compose --env-file .env.dev -f docker-compose.yaml"
# 開発用オーバーライドファイルが存在する場合は追加する
if [ -f "docker-compose.dev.yaml" ]; then
  DC="${DC} -f docker-compose.dev.yaml"
fi

echo "=== k1s0 local environment ==="

case "$PROFILES" in
  infra)
    echo "Starting infrastructure (PostgreSQL, Kafka, Redis, Keycloak)..."
    ${DC} --profile infra up -d
    ;;
  system)
    echo "Starting infrastructure + system servers..."
    ${DC} --profile infra --profile system up -d
    ;;
  all)
    echo "Starting all services (infra + system + business)..."
    ${DC} --profile infra --profile system --profile business up -d
    ;;
  *)
    echo "Usage: $0 [infra|system|all]"
    echo "  infra   — PostgreSQL, Kafka, Redis, Keycloak"
    echo "  system  — infra + system servers (auth, config, saga, dlq, featureflag,"
    echo "             ratelimit, tenant, vault-svc, api-registry, app-registry,"
    echo "             event-monitor, event-store, file, master-maintenance, navigation,"
    echo "             notification, policy, quota, rule-engine, scheduler, search,"
    echo "             service-catalog, session, workflow, ai-gateway, ai-agent,"
    echo "             graphql-gateway, bff-proxy)"
    echo "  all     — infra + system + business tier (default)"
    exit 1
    ;;
esac

echo ""
echo "Waiting for services to become healthy..."
sleep 5

${DC} ps --format "table {{.Name}}\t{{.Status}}\t{{.Ports}}" 2>/dev/null || ${DC} ps

echo ""
echo "=== Health check (readyz) ==="
# system tier — 全サービスのデフォルトポートで /readyz を確認する
for port in 8083 8084 8085 8086 8087 8088 8089 8091 8082 8092 \
            8093 8094 8095 8096 8097 8098 8099 8122 8101 8102 \
            8103 8104 8105 8106 8107 8108 8120 8121; do
  status=$(curl -sf "http://localhost:${port}/readyz" -o /dev/null -w "%{http_code}" 2>/dev/null || echo "000")
  if [ "$status" = "200" ]; then
    echo "  port ${port}: ready (HTTP ${status})"
  else
    echo "  port ${port}: not ready (HTTP ${status})"
  fi
done

if [ "$PROFILES" = "all" ]; then
  # business tier
  for port in 8211 8311 8321 8331; do
    status=$(curl -sf "http://localhost:${port}/readyz" -o /dev/null -w "%{http_code}" 2>/dev/null || echo "000")
    if [ "$status" = "200" ]; then
      echo "  port ${port}: ready (HTTP ${status})"
    else
      echo "  port ${port}: not ready (HTTP ${status})"
    fi
  done
fi

echo ""
echo "Done. Keycloak admin: http://localhost:8180 (admin/dev)"
