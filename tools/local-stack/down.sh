#!/usr/bin/env bash
#
# tools/local-stack/down.sh — kind cluster の破棄（manifest と PV 全消去）
#
# 用途: 開発スタックの完全リセット。データは消える。バックアップが必要なら別途 export を実行。

set -euo pipefail

KIND_CLUSTER_NAME="k1s0-local"

log() { printf '\033[36m[local-stack]\033[0m %s\n' "$*"; }

if kind get clusters 2>/dev/null | grep -q "^${KIND_CLUSTER_NAME}$"; then
    log "kind cluster '${KIND_CLUSTER_NAME}' を削除"
    kind delete cluster --name "${KIND_CLUSTER_NAME}"
else
    log "kind cluster '${KIND_CLUSTER_NAME}' は存在しません"
fi
log "完了"
