#!/usr/bin/env bash
# start-local.sh — k1s0 ローカル開発環境の一括起動
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

PROFILES="${1:-all}"

echo "=== k1s0 local environment ==="

case "$PROFILES" in
  infra)
    echo "Starting infrastructure (PostgreSQL, Kafka, Redis, Keycloak)..."
    docker compose --profile infra up -d
    ;;
  system)
    echo "Starting infrastructure + system servers..."
    docker compose --profile infra --profile system up -d
    ;;
  all)
    echo "Starting all services (infra + system + business)..."
    docker compose --profile infra --profile system --profile business up -d
    ;;
  *)
    echo "Usage: $0 [infra|system|all]"
    echo "  infra   — PostgreSQL, Kafka, Redis, Keycloak"
    echo "  system  — infra + 10 system servers"
    echo "  all     — infra + system + business tier (default)"
    exit 1
    ;;
esac

echo ""
echo "Waiting for services to become healthy..."
sleep 5

docker compose ps --format "table {{.Name}}\t{{.Status}}\t{{.Ports}}" 2>/dev/null || docker compose ps

echo ""
echo "=== Health check ==="
for port in 8083 8084 8085 8086 8087 8088 8089 8091 8092 8082; do
  name=$(curl -sf "http://localhost:${port}/healthz" 2>/dev/null && echo " (port ${port})" || echo "  port ${port}: not ready")
  echo "  ${name}"
done

if [ "$PROFILES" = "all" ]; then
  curl -sf "http://localhost:8210/healthz" > /dev/null 2>&1 && echo "  project-master (port 8210): healthy" || echo "  project-master (port 8210): not ready"
fi

echo ""
echo "Done. Keycloak admin: http://localhost:8180 (admin/admin)"
