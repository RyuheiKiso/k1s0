#!/usr/bin/env bash
#
# tools/local-stack/openbao-dev/up.sh — OpenBao dev server のスタンドアロン起動
#
# 設計書: docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md
# IMP-DEV-DC-015: OpenBao dev server のローカル展開
#
# kind を使わない軽量経路。.devcontainer/.env.local に root token を書き出す。

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
COMPOSE_FILE="${REPO_ROOT}/tools/local-stack/openbao-dev/docker-compose.yml"
ENV_FILE="${REPO_ROOT}/.devcontainer/.env.local"

log() { printf '\033[36m[openbao-dev]\033[0m %s\n' "$*"; }

# k1s0-local network が無ければ作成（compose の external 宣言に対応）
if ! docker network inspect k1s0-local >/dev/null 2>&1; then
    log "docker network 'k1s0-local' を作成"
    docker network create k1s0-local >/dev/null
fi

log "OpenBao dev を起動"
docker compose -f "${COMPOSE_FILE}" up -d openbao-dev openbao-init

mkdir -p "$(dirname "${ENV_FILE}")"
cat > "${ENV_FILE}" <<EOF
# 自動生成: tools/local-stack/openbao-dev/up.sh
# Dev Container から OpenBao dev を呼ぶ際の環境変数。本番では使用しないこと。
BAO_ADDR=http://localhost:8200
BAO_TOKEN=dev-root-token
EOF

log "${ENV_FILE} に dev token を書き出しました（gitignore 必須）"
log "OpenBao UI: http://localhost:8200"
log "停止は: docker compose -f ${COMPOSE_FILE} down"
