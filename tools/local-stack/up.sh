#!/usr/bin/env bash
#
# tools/local-stack/up.sh — kind + 本番再現スタックの一括起動
#
# 設計書: docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md
# IMP-DEV-POL-006: ローカルは kind/k3d + Dapr Local で本番再現する
# IMP-DEV-DC-014: ローカル Kubernetes と Dapr Local の統合
#
# Usage:
#   tools/local-stack/up.sh                       # 既定 role=docs-writer で kind 起動のみ
#   tools/local-stack/up.sh --role <role>         # 役割別配備
#   tools/local-stack/up.sh --layers cni,cert     # 特定レイヤのみ配備
#   tools/local-stack/up.sh --skip backstage      # 特定レイヤをスキップ
#   tools/local-stack/up.sh --no-cluster          # 既存 cluster に対して manifest のみ当てる
#   tools/local-stack/up.sh --observability       # observability レイヤを追加配備
#
# レイヤと依存:
#   cluster -> cni -> cert-manager -> metallb -> istio-ambient -> kyverno -> spire ->
#   dapr -> flagd -> gitea -> registry -> argocd -> argo-rollouts -> envoy-gateway ->
#   cnpg -> kafka -> temporal -> minio -> valkey -> openbao -> backstage ->
#   observability -> keycloak
#
# Mode (ADR-POL-002):
#   --mode dev    開発者の探索を許す。Kyverno の drift policy は audit のみ。既定 (tier*-dev / docs-writer)。
#   --mode strict drift を deny。SoT 違反 (up.sh 既知 release set 外の helm install) を runtime 拒否。
#                  既定 (infra-ops / full / production-mirror)。

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
MANIFESTS="${REPO_ROOT}/tools/local-stack/manifests"
KIND_CONFIG="${REPO_ROOT}/tools/local-stack/kind-cluster.yaml"
KIND_CLUSTER_NAME="k1s0-local"

# Helm chart のピン留めバージョン（2026-04 時点で実存性検証済み）。
# 値の更新は Renovate でなく手動でレビューする方針。
# export しているのは lib/apply-layers.sh が source 後にこれらを参照するため
# （export しないと shellcheck SC2034 で "未使用" 警告が出る）。
export CERT_MANAGER_VERSION="v1.20.2"
export KYVERNO_VERSION="3.5.1"
export SPIRE_CRDS_VERSION="0.5.0"
export SPIRE_VERSION="0.28.4"
export DAPR_VERSION="1.17.5"
export ARGOCD_VERSION="9.5.4"
export CNPG_VERSION="0.28.0"
export STRIMZI_VERSION="0.51.0"
export OPENBAO_VERSION="0.27.2"
export BACKSTAGE_VERSION="2.6.3"
export KEYCLOAK_VERSION="25.2.0"
export VALKEY_VERSION="5.5.1"
export LOKI_CHART_VERSION="6.21.0"
export TEMPO_CHART_VERSION="1.24.4"
export GRAFANA_CHART_VERSION="8.10.4"
export OTEL_COLLECTOR_VERSION="0.117.0"
export PROMETHEUS_CHART_VERSION="29.4.0"
export METALLB_VERSION="v0.14.9"
export CALICO_VERSION="v3.29.1"
# ADR-POL-002 で追加された SoT 強制対象の新規レイヤ
export ARGO_ROLLOUTS_VERSION="2.40.9"
export ENVOY_GATEWAY_VERSION="v1.2.4"
export TEMPORAL_VERSION="0.65.0"

declare -A ROLE_LAYERS=(
    ["tier1-rust-dev"]="cni cert-manager metallb istio kyverno spire dapr flagd gitea registry argocd argo-rollouts envoy-gateway cnpg kafka minio valkey openbao"
    ["tier1-go-dev"]="cni cert-manager metallb istio kyverno spire dapr flagd gitea registry argocd argo-rollouts envoy-gateway cnpg kafka minio valkey openbao"
    ["tier2-dev"]="cni cert-manager metallb istio kyverno spire dapr flagd gitea registry argocd argo-rollouts cnpg kafka temporal minio valkey openbao"
    ["tier3-web-dev"]="cni cert-manager metallb istio kyverno spire dapr flagd gitea registry argocd argo-rollouts cnpg openbao backstage"
    ["tier3-native-dev"]="cni cert-manager metallb istio kyverno spire dapr flagd gitea registry argocd argo-rollouts cnpg openbao"
    ["platform-cli-dev"]="cni cert-manager metallb istio kyverno spire dapr flagd gitea registry argocd argo-rollouts envoy-gateway cnpg backstage"
    ["sdk-dev"]="cni cert-manager metallb istio kyverno spire dapr flagd cnpg kafka openbao"
    ["infra-ops"]="cni cert-manager metallb istio kyverno spire dapr flagd gitea registry argocd argo-rollouts envoy-gateway cnpg kafka temporal minio valkey openbao backstage observability keycloak"
    ["docs-writer"]="cni cert-manager"
    ["full"]="cni cert-manager metallb istio kyverno spire dapr flagd gitea registry argocd argo-rollouts envoy-gateway cnpg kafka temporal minio valkey openbao backstage observability keycloak"
)

# ADR-POL-002: role 別の既定 mode（drift policy の Kyverno 強制度）。
# `--mode` で明示指定された場合はそちらが優先。
declare -A ROLE_MODE=(
    ["tier1-rust-dev"]="dev"
    ["tier1-go-dev"]="dev"
    ["tier2-dev"]="dev"
    ["tier3-web-dev"]="dev"
    ["tier3-native-dev"]="dev"
    ["platform-cli-dev"]="dev"
    ["sdk-dev"]="dev"
    ["docs-writer"]="dev"
    ["infra-ops"]="strict"
    ["full"]="strict"
)

ROLE="docs-writer"
SELECTED_LAYERS=""
SKIP_LAYERS=""
SKIP_CLUSTER=0
EXTRA_OBS=0
MODE=""  # ADR-POL-002: dev | strict（未指定なら ROLE_MODE のデフォルトを使う）

log() { printf '\033[36m[local-stack]\033[0m %s\n' "$*"; }
warn() { printf '\033[33m[local-stack][warn]\033[0m %s\n' "$*"; }
fail() { printf '\033[31m[local-stack][error]\033[0m %s\n' "$*" >&2; exit 1; }

usage() {
    sed -n '3,20p' "$0" | sed 's/^# \{0,1\}//'
    exit 1
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help) usage ;;
        --role) ROLE="$2"; shift 2 ;;
        --layers) SELECTED_LAYERS="$2"; shift 2 ;;
        --skip) SKIP_LAYERS="$2"; shift 2 ;;
        --no-cluster) SKIP_CLUSTER=1; shift ;;
        --observability) EXTRA_OBS=1; shift ;;
        --mode) MODE="$2"; shift 2 ;;
        *) fail "未知の引数: $1" ;;
    esac
done

if [[ -z "${ROLE_LAYERS[${ROLE}]+x}" ]]; then
    fail "未知の role: ${ROLE}（利用可能: ${!ROLE_LAYERS[*]}）"
fi

# mode 解決（明示指定 > role の既定 > "dev" fallback）
if [[ -z "${MODE}" ]]; then
    MODE="${ROLE_MODE[${ROLE}]:-dev}"
fi
if [[ "${MODE}" != "dev" && "${MODE}" != "strict" ]]; then
    fail "未知の mode: ${MODE}（dev | strict のみ）"
fi

if [[ -n "${SELECTED_LAYERS}" ]]; then
    LAYERS_TO_APPLY="${SELECTED_LAYERS//,/ }"
else
    LAYERS_TO_APPLY="${ROLE_LAYERS[${ROLE}]}"
fi
[[ "${EXTRA_OBS}" == "1" ]] && LAYERS_TO_APPLY="${LAYERS_TO_APPLY} observability"

if [[ -n "${SKIP_LAYERS}" ]]; then
    for s in ${SKIP_LAYERS//,/ }; do
        LAYERS_TO_APPLY="${LAYERS_TO_APPLY//${s}/}"
    done
fi
LAYERS_TO_APPLY="$(echo "${LAYERS_TO_APPLY}" | tr -s ' ')"

log "role=${ROLE}"
log "mode=${MODE} (ADR-POL-002)"
log "layers=${LAYERS_TO_APPLY}"

has_layer() {
    local needle="$1"
    [[ " ${LAYERS_TO_APPLY} " == *" ${needle} "* ]]
}

ensure_helm_repo() {
    local name="$1"; local url="$2"
    helm repo list 2>/dev/null | grep -q "^${name}\b" || helm repo add "${name}" "${url}" >/dev/null
    helm repo update "${name}" >/dev/null 2>&1 || true
}

wait_for_pods_ready() {
    local ns="$1"; local timeout="${2:-300s}"
    log "waiting for pods in ns=${ns} (timeout=${timeout})"
    kubectl -n "${ns}" wait --for=condition=Ready pods --all --timeout="${timeout}" 2>/dev/null \
        || warn "ns=${ns} の一部 pod が timeout 内に Ready になりませんでした（先に進みます）"
}

# kind の docker network から動的に subnet を取得し、MetalLB IPAddressPool を生成する。
# 固定値はホストの docker daemon が異なる subnet を割当てた場合に失敗するため、
# `docker network inspect` 経由で実際の subnet を抽出する。
generate_metallb_pool() {
    # ADR-POL-002 P4 で発覚: docker network kind が IPv4 と IPv6 両方の subnet を持つ環境では
    # IPAM.Config[0] が IPv6 になる場合があり、"fc00:f853:ccd:e793::/64" のような subnet を引いて
    # MetalLB の IPv4 形式 IPAddressPool 生成が壊れる。明示的に IPv4 subnet のみを抽出する。
    local subnet
    subnet="$(docker network inspect kind 2>/dev/null \
        | python3 -c "import sys,json
d = json.load(sys.stdin)
for c in d[0]['IPAM']['Config']:
    s = c.get('Subnet','')
    if '.' in s and ':' not in s:
        print(s)
        break" 2>/dev/null \
        || echo "")"
    if [[ -z "${subnet}" ]]; then
        warn "kind docker network から IPv4 subnet を取得できませんでした。固定値 172.18.255.200-250 を使用"
        echo "172.18.255.200-172.18.255.250"
        return
    fi
    local prefix
    prefix="$(echo "${subnet}" | cut -d. -f1-2)"
    echo "${prefix}.255.200-${prefix}.255.250"
}

start_cluster() {
    if [[ "${SKIP_CLUSTER}" == "1" ]]; then
        log "skip cluster (--no-cluster)"
        return 0
    fi
    if kind get clusters 2>/dev/null | grep -q "^${KIND_CLUSTER_NAME}$"; then
        log "kind cluster '${KIND_CLUSTER_NAME}' は既に存在します"
    else
        log "kind cluster '${KIND_CLUSTER_NAME}' を作成"
        kind create cluster --config "${KIND_CONFIG}"
    fi
    kubectl cluster-info --context "kind-${KIND_CLUSTER_NAME}"
    log "namespace 群を作成"
    kubectl apply -f "${MANIFESTS}/00-namespaces.yaml"
}


# ADR-POL-002 P4 finishing で 500 行制限超過のため apply_* 関数を分離。
# 親 up.sh の readonly / 関数（log/warn/has_layer/ensure_helm_repo 等）を継承する。
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/apply-layers.sh
source "${SCRIPT_DIR}/lib/apply-layers.sh"


start_cluster
apply_cni
apply_cert_manager
apply_metallb
apply_istio
apply_kyverno
apply_spire
apply_dapr
apply_flagd
apply_gitea
apply_registry
apply_argocd
apply_argo_rollouts
apply_envoy_gateway
apply_cnpg
apply_kafka
apply_temporal
apply_minio
apply_valkey
apply_openbao
apply_backstage
apply_observability
apply_keycloak

log ""
log "=========================================="
log "  k1s0 local-stack 起動完了 (role=${ROLE} mode=${MODE})"
log "=========================================="
log "アクセスポイント (kind の hostPort 経由 / port-forward):"
log "  - Argo CD UI:   http://localhost:30080  (admin / 初期 PW は kubectl -n argocd get secret argocd-initial-admin-secret)"
log "  - Backstage:    http://localhost:30700"
log "  - Grafana:      http://localhost:30300  (admin / k1s0-local-dev-password)"
log "  - Gitea:        kubectl -n gitops port-forward svc/gitea 3000:3000  (admin / argocd:ArgoCD123!)"
log "  - Temporal Web: kubectl -n temporal port-forward svc/temporal-web 8080:8080"
log ""
log "次の操作: tools/local-stack/status.sh で配備状態を確認、down.sh で停止"
log "再構築前のバックアップ: tools/local-stack/backup-cluster.sh (ADR-POL-002)"
log ""
if [[ "${MODE}" == "strict" ]]; then
    log "Mode=strict: Kyverno で SoT 違反 (up.sh 既知 release set 外の helm install) を deny"
else
    log "Mode=dev: Kyverno は audit のみ。drift は警告（PR 段階で CI が検出）"
fi
