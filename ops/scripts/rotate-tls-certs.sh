#!/usr/bin/env bash
# =============================================================================
# ops/scripts/rotate-tls-certs.sh — TLS 証明書一括ローテーション
#
# 役割: cert-manager / SPIRE / Strimzi の証明書を一括強制更新する。
#       通常は cert-manager auto-renew に任せるが、緊急ローテーション時に使用。
# 関連 Runbook: RB-SEC-002
#
# Usage:
#   ops/scripts/rotate-tls-certs.sh \
#     --target all|cert-manager|spire|strimzi \
#     [--dry-run]
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "${SCRIPT_DIR}/lib/common.sh"

TARGET="all"
DRY_RUN=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --target) TARGET="$2"; shift 2 ;;
        --dry-run) DRY_RUN=1; shift ;;
        -h|--help) sed -n '3,12p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "[error] 未知: $1"; exit 2 ;;
    esac
done

LOG_FILE="${TMPDIR:-/tmp}/k1s0-rotate-tls-$(k1s0_utc_timestamp).log"

k1s0_log info "TLS 証明書 rotation 開始 target=${TARGET} dry_run=${DRY_RUN}"
k1s0_need kubectl
k1s0_assert_prod

run() {
    [[ "${DRY_RUN}" == "1" ]] && k1s0_log info "[dry-run] $*" || eval "$@"
}

rotate_cert_manager() {
    k1s0_log info "cert-manager: 全 Certificate を再発行"
    # Certificate を一覧して順次削除（cert-manager が自動再発行）
    kubectl get certificate -A -o json \
        | jq -r '.items[] | "\(.metadata.namespace) \(.metadata.name)"' \
        | while read -r ns name; do
            run "kubectl annotate certificate ${name} -n ${ns} cert-manager.io/issue-temporary-certificate=true --overwrite"
        done
}

rotate_spire() {
    k1s0_log info "SPIRE: Server / Agent rolling restart"
    run "kubectl rollout restart deployment/spire-server -n spire-system"
    run "kubectl rollout status deployment/spire-server -n spire-system --timeout=300s"
    run "kubectl rollout restart daemonset/spire-agent -n spire-system"
    run "kubectl rollout status daemonset/spire-agent -n spire-system --timeout=300s"
}

rotate_strimzi() {
    k1s0_log info "Strimzi: Kafka cluster-ca 強制再発行"
    run "kubectl annotate secret k1s0-kafka-cluster-ca-cert strimzi.io/force-renew=true -n kafka --overwrite"
    run "kubectl annotate secret k1s0-kafka-clients-ca-cert strimzi.io/force-renew=true -n kafka --overwrite"
    k1s0_log info "broker rolling restart は Strimzi Operator が自動実施"
}

case "${TARGET}" in
    cert-manager) rotate_cert_manager ;;
    spire) rotate_spire ;;
    strimzi) rotate_strimzi ;;
    all) rotate_cert_manager; rotate_spire; rotate_strimzi ;;
    *) echo "[error] 未対応 target: ${TARGET}"; exit 2 ;;
esac

k1s0_log info "rotation 完了。検証: ops/runbooks/incidents/RB-SEC-002-cert-expiry.md §6 を実施"
