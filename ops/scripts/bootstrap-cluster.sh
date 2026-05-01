#!/usr/bin/env bash
# =============================================================================
# ops/scripts/bootstrap-cluster.sh — 新規 k8s クラスタの初期セットアップ
#
# 役割: 新規 GKE/EKS クラスタに対し、ArgoCD + cert-manager + cnpg-operator +
#       strimzi-operator + spire を bootstrap する。
#       本番災害復旧の Phase 1 でも利用される（rebuild-cluster-from-scratch.sh と分担）。
#
# Usage:
#   ops/scripts/bootstrap-cluster.sh [--dry-run]
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/common.sh
. "${SCRIPT_DIR}/lib/common.sh"

DRY_RUN=0
[[ "${1:-}" == "--dry-run" ]] && DRY_RUN=1

LOG_FILE="${TMPDIR:-/tmp}/k1s0-bootstrap-$(k1s0_utc_timestamp).log"

k1s0_log info "クラスタ bootstrap 開始 dry_run=${DRY_RUN}"
k1s0_need kubectl argocd helm

# kubectl context 確認（誤実行防止: prod 以外でも本スクリプトは利用可、staging で先行実施想定）
k1s0_log info "current context: $(kubectl config current-context)"

apply() {
    if [[ "${DRY_RUN}" == "1" ]]; then
        k1s0_log info "[dry-run] $*"
    else
        eval "$@"
    fi
}

# Step 1: ArgoCD インストール
k1s0_log info "Step 1: ArgoCD インストール"
apply "kubectl create namespace argocd 2>/dev/null || true"
apply "kubectl apply -n argocd -f https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml"
apply "kubectl wait --for=condition=Available deploy/argocd-server -n argocd --timeout=300s"

# Step 2: App of Apps 適用
k1s0_log info "Step 2: App of Apps 適用"
apply "kubectl apply -f infra/k8s/bootstrap/app-of-apps.yaml -n argocd"

# Step 3: 主要 Operator が起動するまで待機
k1s0_log info "Step 3: 主要 Operator の起動待ち（最大 10 分）"
for ns in cert-manager cnpg-system kafka spire-system openbao; do
    apply "kubectl wait --for=condition=Available deployment --all -n ${ns} --timeout=600s 2>/dev/null || true"
done

k1s0_log info "bootstrap 完了"
k1s0_log info "次のステップ: ArgoCD UI で全 Application が Synced & Healthy であることを確認"
