#!/usr/bin/env bash
# =============================================================================
# ops/scripts/harvest-logs.sh — インシデント対応用ログ収集
#
# 役割: 指定期間の Pod ログ + イベント + Loki クエリ結果を tar に固める。
#       ポストモーテム / 法的開示対応 / 監督官庁報告用。
# 関連 Runbook: RB-COMP-001（法的開示対応）/ RB-SEC-005（PII 漏えい）
#
# Usage:
#   ops/scripts/harvest-logs.sh \
#     --since 2h \
#     --namespace k1s0-tier1 \
#     [--out /tmp/harvest.tar.gz] \
#     [--dry-run]
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "${SCRIPT_DIR}/lib/common.sh"

SINCE="2h"
NAMESPACE=""
OUT="/tmp/k1s0-harvest-$(k1s0_utc_timestamp).tar.gz"
DRY_RUN=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --since) SINCE="$2"; shift 2 ;;
        --namespace) NAMESPACE="$2"; shift 2 ;;
        --out) OUT="$2"; shift 2 ;;
        --dry-run) DRY_RUN=1; shift ;;
        -h|--help) sed -n '3,15p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "[error] 未知: $1"; exit 2 ;;
    esac
done

if [[ -z "${NAMESPACE}" ]]; then
    echo "[error] --namespace は必須"; exit 2
fi

WORKDIR=$(mktemp -d /tmp/k1s0-harvest.XXXXXX)
trap 'rm -rf "${WORKDIR}"' EXIT

LOG_FILE="${WORKDIR}/harvest.log"
k1s0_log info "log 収集開始 namespace=${NAMESPACE} since=${SINCE} out=${OUT}"
k1s0_need kubectl logcli sha256sum

run() {
    [[ "${DRY_RUN}" == "1" ]] && k1s0_log info "[dry-run] $*" || eval "$@"
}

# Pod logs
k1s0_log info "Pod logs 取得"
run "kubectl get pods -n ${NAMESPACE} -o name > ${WORKDIR}/pods.list"
while read -r pod; do
    pod_name=${pod##*/}
    run "kubectl logs --since=${SINCE} -n ${NAMESPACE} ${pod_name} > ${WORKDIR}/${pod_name}.log 2>&1 || true"
done < "${WORKDIR}/pods.list"

# Events
k1s0_log info "Events 取得"
run "kubectl get events -n ${NAMESPACE} --sort-by='.lastTimestamp' > ${WORKDIR}/events.txt"

# Loki クエリ（採用後の運用拡大時 で logcli alias 設定済みであれば）
if command -v logcli >/dev/null; then
    k1s0_log info "Loki クエリ実行"
    SINCE_FMT=$(date -u -d "${SINCE} ago" -Iseconds 2>/dev/null || echo "")
    if [[ -n "${SINCE_FMT}" ]]; then
        run "logcli query '{namespace=\"${NAMESPACE}\"} | json' --since=${SINCE} --output=jsonl > ${WORKDIR}/loki.jsonl"
    fi
fi

# tar.gz
k1s0_log info "tar.gz 生成"
run "tar czf ${OUT} -C ${WORKDIR} ."
run "sha256sum ${OUT} > ${OUT}.sha256"

k1s0_log info "完了: ${OUT} (sha256: ${OUT}.sha256)"
k1s0_log info "法的開示対応の場合は GPG で暗号化してから提供すること（RB-COMP-001 §Step 3 参照）"
