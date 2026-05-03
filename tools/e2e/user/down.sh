#!/usr/bin/env bash
#
# tools/e2e/user/down.sh — user suite cluster の削除（kind delete cluster）
#
# 設計正典:
#   ADR-TEST-008（user suite cluster ライフサイクル）
#   docs/05_実装/30_CI_CD設計/35_e2e_test_design/20_user_suite/03_Makefile_target.md

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIB_DIR="$(cd "${SCRIPT_DIR}/../lib" && pwd)"

# shellcheck source=../lib/common.sh
source "${LIB_DIR}/common.sh"

KIND_CLUSTER_NAME="k1s0-user-e2e"
PURGE_ARTIFACTS=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --purge-artifacts) PURGE_ARTIFACTS=1; shift ;;
        --keep-artifacts) PURGE_ARTIFACTS=0; shift ;;
        -h|--help)
            sed -n '2,10p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *) e2e_warn "未知の引数: $1"; exit 2 ;;
    esac
done

# kind 不在時は warn のみ（既に撤去済の可能性）
if ! command -v kind >/dev/null 2>&1; then
    e2e_warn "kind がない、削除 skip"
    exit 0
fi

# 既存 cluster があれば削除
if kind get clusters 2>/dev/null | grep -q "^${KIND_CLUSTER_NAME}$"; then
    e2e_log "kind cluster '${KIND_CLUSTER_NAME}' 削除"
    kind delete cluster --name "${KIND_CLUSTER_NAME}"
else
    e2e_log "kind cluster '${KIND_CLUSTER_NAME}' は不在"
fi

# artifact ディレクトリの扱い
if [[ "${PURGE_ARTIFACTS}" -eq 1 ]]; then
    e2e_warn "--purge-artifacts: tests/.user-e2e/ 全削除"
    rm -rf "$(e2e_repo_root)/tests/.user-e2e/"
fi

e2e_log "user suite cluster 削除完了"
