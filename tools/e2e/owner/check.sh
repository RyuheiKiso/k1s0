#!/usr/bin/env bash
#
# tools/e2e/owner/check.sh — owner suite cluster の起動状態確認
#
# 設計正典:
#   ADR-TEST-008
#   docs/05_実装/30_CI_CD設計/35_e2e_test_design/10_owner_suite/03_Makefile_target.md
#
# Usage:
#   tools/e2e/owner/check.sh
#   tools/e2e/owner/check.sh --vm-prefix PFX
#
# 終了コード:
#   0 = cluster 健全（全 5 VM Running + kubeconfig context 一致 + 全 node Ready）
#   1 = いずれかの状態が未達

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIB_DIR="$(cd "${SCRIPT_DIR}/../lib" && pwd)"

# shellcheck source=../lib/common.sh
source "${LIB_DIR}/common.sh"
# shellcheck source=../lib/multipass.sh
source "${LIB_DIR}/multipass.sh"

VM_PREFIX="k1s0-owner"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --vm-prefix) VM_PREFIX="$2"; shift 2 ;;
        --vm-prefix=*) VM_PREFIX="${1#*=}"; shift ;;
        -h|--help)
            sed -n '2,12p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *) e2e_warn "未知の引数: $1"; exit 2 ;;
    esac
done

# multipass / kubectl 不在時は早期 fail
e2e_require_bin multipass || exit 1
e2e_require_bin kubectl || exit 1

# 5 VM すべての Running 状態を確認
ALL_VMS=(
    "${VM_PREFIX}-cp-1"
    "${VM_PREFIX}-cp-2"
    "${VM_PREFIX}-cp-3"
    "${VM_PREFIX}-w-1"
    "${VM_PREFIX}-w-2"
)
not_running=0
for vm in "${ALL_VMS[@]}"; do
    if e2e_multipass_running "${vm}"; then
        e2e_log "  ${vm}: Running"
    else
        e2e_warn "  ${vm}: 不在 / Stopped"
        not_running=$((not_running + 1))
    fi
done
if [[ "${not_running}" -gt 0 ]]; then
    e2e_warn "${not_running} 件の VM が Running でない、./tools/e2e/owner/up.sh で起動してください"
    exit 1
fi

# kubeconfig context 確認（owner suite の context = k1s0-owner-e2e）
if ! e2e_kubectl_context_check k1s0-owner-e2e; then
    e2e_warn "kubeconfig context を切り替えてください: kubectl config use-context k1s0-owner-e2e"
    exit 1
fi

# 全 node Ready 確認
if ! e2e_wait_nodes_ready 60; then
    e2e_warn "node Ready 確認失敗、./tools/e2e/owner/up.sh の再実行を検討してください"
    exit 1
fi

e2e_log "owner suite cluster 健全（5 VM Running + context = k1s0-owner-e2e + 全 node Ready）"
exit 0
