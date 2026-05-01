#!/usr/bin/env bash
# =============================================================================
# ops/dr/scripts/rebuild-cluster-from-scratch.sh — RB-DR-001 自動化
#
# 設計: ops/dr/scenarios/RB-DR-001-cluster-rebuild.md の Phase 1〜3 を 1 コマンド化
# 関連: NFR-A-REC-001（RTO 4h）、FMEA-008
#
# 役割:
#   災害発生時、Cloud Provider CLI + ArgoCD で本番クラスタをゼロから再構築する。
#   Phase 1（k8s 作成）→ Phase 2（CNPG リストア）→ Phase 3（アプリ復旧）を順次実行。
#
# Usage:
#   ops/dr/scripts/rebuild-cluster-from-scratch.sh \
#     --provider gke \
#     --region asia-northeast1 \
#     --cluster-name k1s0-dr \
#     --backup-name k1s0-postgres-backup-latest \
#     [--dry-run]
#
# 環境変数:
#   GCLOUD_PROJECT       — GCP project ID
#   AWS_REGION           — AWS リージョン（provider=eks の場合）
#   ARGOCD_AUTH_TOKEN    — ArgoCD 認証
#   MINIO_ENDPOINT       — Barman archive 取得元
#
# Exit code:
#   0 — Phase 1〜3 完了
#   1 — Phase 失敗
#   2 — 引数エラー / 必須ツール不在
# =============================================================================
set -euo pipefail

PROVIDER=""
REGION=""
CLUSTER_NAME="k1s0-dr"
BACKUP_NAME=""
DRY_RUN=0

usage() {
    sed -n '3,30p' "$0" | sed 's/^# \{0,1\}//'
    exit 2
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --provider) PROVIDER="$2"; shift 2 ;;
        --region) REGION="$2"; shift 2 ;;
        --cluster-name) CLUSTER_NAME="$2"; shift 2 ;;
        --backup-name) BACKUP_NAME="$2"; shift 2 ;;
        --dry-run) DRY_RUN=1; shift ;;
        -h|--help) usage ;;
        *) echo "[error] 未知のオプション: $1" >&2; usage ;;
    esac
done

if [[ -z "${PROVIDER}" || -z "${REGION}" || -z "${BACKUP_NAME}" ]]; then
    echo "[error] --provider / --region / --backup-name は必須" >&2
    usage
fi

LOG_DIR="${TMPDIR:-/tmp}/k1s0-dr-rebuild"
mkdir -p "${LOG_DIR}"
LOG_FILE="${LOG_DIR}/${CLUSTER_NAME}-$(date -u +%Y%m%dT%H%M%SZ).log"
START_EPOCH=$(date +%s)

log() {
    local lvl="$1"; shift
    printf '[%s] [%s] %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "${lvl}" "$*" \
      | tee -a "${LOG_FILE}"
}

elapsed_min() {
    awk -v s="${START_EPOCH}" -v n="$(date +%s)" 'BEGIN{printf "%.1f", (n-s)/60}'
}

# ---------------------------------------------------------------------------
# Phase 1: k8s クラスタ作成 + ArgoCD ブートストラップ（〜60 分）
# ---------------------------------------------------------------------------
phase1_k8s_bootstrap() {
    log info "Phase 1 開始: k8s クラスタ作成 (provider=${PROVIDER})"

    if [[ "${DRY_RUN}" == "1" ]]; then
        log info "[dry-run] gcloud/aws CLI で k8s クラスタを作成"
        log info "[dry-run] kubectl apply infra/k8s/bootstrap/app-of-apps.yaml"
        return 0
    fi

    case "${PROVIDER}" in
        gke)
            gcloud container clusters create "${CLUSTER_NAME}" \
                --region "${REGION}" \
                --num-nodes 3 \
                --machine-type n2-standard-4 >>"${LOG_FILE}" 2>&1
            gcloud container clusters get-credentials "${CLUSTER_NAME}" \
                --region "${REGION}"
            ;;
        eks)
            eksctl create cluster \
                --name "${CLUSTER_NAME}" \
                --region "${REGION}" \
                --nodes 3 \
                --node-type t3.xlarge >>"${LOG_FILE}" 2>&1
            ;;
        *)
            log error "未対応 provider: ${PROVIDER}"
            return 1
            ;;
    esac

    log info "ArgoCD ブートストラップ"
    kubectl create namespace argocd >>"${LOG_FILE}" 2>&1 || true
    kubectl apply -n argocd \
        -f https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml \
        >>"${LOG_FILE}" 2>&1
    kubectl wait --for=condition=Available deploy/argocd-server -n argocd --timeout=300s

    log info "App of Apps 適用"
    kubectl apply -f infra/k8s/bootstrap/app-of-apps.yaml -n argocd >>"${LOG_FILE}" 2>&1
    argocd app sync infra-root --prune --timeout 600 >>"${LOG_FILE}" 2>&1
    log info "Phase 1 完了 (経過 $(elapsed_min) 分)"
}

# ---------------------------------------------------------------------------
# Phase 2: CNPG リストア（〜90 分）
# ---------------------------------------------------------------------------
phase2_cnpg_restore() {
    log info "Phase 2 開始: CNPG リストア (backup=${BACKUP_NAME})"
    if [[ "${DRY_RUN}" == "1" ]]; then
        log info "[dry-run] CNPG Cluster.spec.bootstrap.recovery で Barman リストア"
        return 0
    fi

    cat <<EOF | kubectl apply -f - >>"${LOG_FILE}" 2>&1
apiVersion: postgresql.cnpg.io/v1
kind: Cluster
metadata:
  name: k1s0-postgres
  namespace: cnpg-system
spec:
  instances: 3
  bootstrap:
    recovery:
      backup:
        name: ${BACKUP_NAME}
  externalClusters:
    - name: ${BACKUP_NAME}
      barmanObjectStore:
        destinationPath: s3://k1s0-postgres-backup/
        endpointURL: ${MINIO_ENDPOINT:-http://minio.minio.svc:9000}
        s3Credentials:
          accessKeyId:
            name: k1s0-minio-credentials
            key: access-key
          secretAccessKey:
            name: k1s0-minio-credentials
            key: secret-key
EOF

    kubectl wait --for=condition=Ready cluster/k1s0-postgres -n cnpg-system --timeout=1800s >>"${LOG_FILE}" 2>&1
    log info "Phase 2 完了 (経過 $(elapsed_min) 分)"
}

# ---------------------------------------------------------------------------
# Phase 3: アプリケーション復旧（〜30 分）
# ---------------------------------------------------------------------------
phase3_app_recovery() {
    log info "Phase 3 開始: アプリケーション復旧"
    if [[ "${DRY_RUN}" == "1" ]]; then
        log info "[dry-run] argocd app sync tier1/tier2/tier3 + spire"
        return 0
    fi

    for app in tier1-services tier2-services tier3-services spire; do
        log info "argocd app sync ${app}"
        argocd app sync "${app}" --prune --timeout 600 >>"${LOG_FILE}" 2>&1 || \
            log warn "${app} sync 失敗 (継続)"
    done

    kubectl rollout status daemonset/spire-agent -n spire-system --timeout=300s
    log info "Phase 3 完了 (経過 $(elapsed_min) 分)"
}

main() {
    log info "DR rebuild 開始: cluster=${CLUSTER_NAME} provider=${PROVIDER} backup=${BACKUP_NAME} dry_run=${DRY_RUN}"
    phase1_k8s_bootstrap || { log error "Phase 1 失敗"; exit 1; }
    phase2_cnpg_restore || { log error "Phase 2 失敗"; exit 1; }
    phase3_app_recovery || { log error "Phase 3 失敗"; exit 1; }

    local d=$(elapsed_min)
    log info "DR rebuild 完了 (合計 ${d} 分)"
    if awk -v d="${d}" 'BEGIN{exit !(d > 240)}'; then
        log warn "RTO 4h 超過 — Phase 4 (DNS 切替) は手動で実施し、ポストモーテムで原因分析"
    fi
}

main "$@"
