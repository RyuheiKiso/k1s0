#!/usr/bin/env bash
#
# tools/e2e/owner/down.sh — owner suite cluster の削除（multipass × 5 一括 purge）
#
# 設計正典:
#   ADR-TEST-008（owner suite cluster ライフサイクル）
#   docs/05_実装/30_CI_CD設計/35_e2e_test_design/10_owner_suite/03_Makefile_target.md
#
# Usage:
#   tools/e2e/owner/down.sh                       # 既定 vm-prefix で削除
#   tools/e2e/owner/down.sh --vm-prefix PFX       # 別 prefix の VM を削除
#   tools/e2e/owner/down.sh --keep-artifacts      # tests/.owner-e2e/<日付>/ は残す（既定）

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIB_DIR="$(cd "${SCRIPT_DIR}/../lib" && pwd)"

# shellcheck source=../lib/common.sh
source "${LIB_DIR}/common.sh"
# shellcheck source=../lib/multipass.sh
source "${LIB_DIR}/multipass.sh"

VM_PREFIX="k1s0-owner"
PURGE_ARTIFACTS=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --vm-prefix) VM_PREFIX="$2"; shift 2 ;;
        --vm-prefix=*) VM_PREFIX="${1#*=}"; shift ;;
        --purge-artifacts) PURGE_ARTIFACTS=1; shift ;;
        --keep-artifacts) PURGE_ARTIFACTS=0; shift ;;
        -h|--help)
            sed -n '2,15p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *) e2e_warn "未知の引数: $1"; exit 2 ;;
    esac
done

# multipass が無い環境では fail せず warn のみ（既に環境を撤去した後など）
if ! command -v multipass >/dev/null 2>&1; then
    e2e_warn "multipass がない、削除 skip（既に撤去済の可能性）"
    exit 0
fi

# 5 VM をまとめて削除
ALL_VMS=(
    "${VM_PREFIX}-cp-1"
    "${VM_PREFIX}-cp-2"
    "${VM_PREFIX}-cp-3"
    "${VM_PREFIX}-w-1"
    "${VM_PREFIX}-w-2"
)
e2e_multipass_delete_all "${ALL_VMS[@]}"

# artifact ディレクトリの扱い（既定は残す）
if [[ "${PURGE_ARTIFACTS}" -eq 1 ]]; then
    e2e_warn "--purge-artifacts: tests/.owner-e2e/ を全削除"
    rm -rf "$(e2e_repo_root)/tests/.owner-e2e/"
else
    e2e_log "artifact は維持: $(e2e_repo_root)/tests/.owner-e2e/"
fi

e2e_log "owner suite cluster 削除完了"
