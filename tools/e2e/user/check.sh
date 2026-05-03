#!/usr/bin/env bash
#
# tools/e2e/user/check.sh — user suite cluster の起動状態確認

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIB_DIR="$(cd "${SCRIPT_DIR}/../lib" && pwd)"

# shellcheck source=../lib/common.sh
source "${LIB_DIR}/common.sh"

KIND_CLUSTER_NAME="k1s0-user-e2e"

while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help)
            echo "Usage: tools/e2e/user/check.sh"
            exit 0
            ;;
        *) e2e_warn "未知の引数: $1"; exit 2 ;;
    esac
done

e2e_require_bin kind || exit 1
e2e_require_bin kubectl || exit 1

# kind cluster 存在確認
if ! kind get clusters 2>/dev/null | grep -q "^${KIND_CLUSTER_NAME}$"; then
    e2e_warn "kind cluster '${KIND_CLUSTER_NAME}' 不在、./tools/e2e/user/up.sh を実行してください"
    exit 1
fi

# kubeconfig context 確認
if ! e2e_kubectl_context_check "kind-${KIND_CLUSTER_NAME}"; then
    e2e_warn "kubeconfig context 切替: kubectl config use-context kind-${KIND_CLUSTER_NAME}"
    exit 1
fi

# node Ready 確認
if ! e2e_wait_nodes_ready 60; then
    e2e_warn "node Ready 失敗、cluster 再起動を検討"
    exit 1
fi

e2e_log "user suite cluster 健全 (kind-${KIND_CLUSTER_NAME})"
exit 0
