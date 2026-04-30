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
readonly CERT_MANAGER_VERSION="v1.20.2"
readonly KYVERNO_VERSION="3.5.1"
readonly SPIRE_CRDS_VERSION="0.5.0"
readonly SPIRE_VERSION="0.28.4"
readonly DAPR_VERSION="1.17.5"
readonly ARGOCD_VERSION="9.5.4"
readonly CNPG_VERSION="0.28.0"
readonly STRIMZI_VERSION="0.51.0"
readonly OPENBAO_VERSION="0.27.2"
readonly BACKSTAGE_VERSION="2.6.3"
readonly KEYCLOAK_VERSION="25.2.0"
readonly VALKEY_VERSION="5.5.1"
readonly LOKI_CHART_VERSION="6.21.0"
readonly TEMPO_CHART_VERSION="1.24.4"
readonly GRAFANA_CHART_VERSION="8.10.4"
readonly OTEL_COLLECTOR_VERSION="0.117.0"
readonly PROMETHEUS_CHART_VERSION="29.4.0"
readonly METALLB_VERSION="v0.14.9"
readonly CALICO_VERSION="v3.29.1"
# ADR-POL-002 で追加された SoT 強制対象の新規レイヤ
readonly ARGO_ROLLOUTS_VERSION="2.40.9"
readonly ENVOY_GATEWAY_VERSION="v1.2.4"
readonly TEMPORAL_VERSION="0.65.0"

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
    local subnet
    subnet="$(docker network inspect kind 2>/dev/null \
        | python3 -c "import sys,json; d=json.load(sys.stdin); print(d[0]['IPAM']['Config'][0]['Subnet'])" 2>/dev/null \
        || echo "")"
    if [[ -z "${subnet}" ]]; then
        warn "kind docker network から subnet を取得できませんでした。固定値 172.18.255.200-250 を使用"
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

apply_cni() {
    has_layer cni || return 0
    log "Calico CNI install (${CALICO_VERSION})"
    kubectl create -f "https://raw.githubusercontent.com/projectcalico/calico/${CALICO_VERSION}/manifests/tigera-operator.yaml" 2>/dev/null || true
    kubectl apply -f - <<EOF || true
apiVersion: operator.tigera.io/v1
kind: Installation
metadata:
  name: default
spec:
  calicoNetwork:
    ipPools:
      - blockSize: 26
        cidr: 10.244.0.0/16
        encapsulation: VXLANCrossSubnet
        natOutgoing: Enabled
        nodeSelector: all()
EOF
    wait_for_pods_ready calico-system 300s
}

apply_cert_manager() {
    has_layer cert-manager || return 0
    log "cert-manager install (${CERT_MANAGER_VERSION})"
    ensure_helm_repo jetstack https://charts.jetstack.io
    helm upgrade --install cert-manager jetstack/cert-manager \
        --namespace cert-manager --version "${CERT_MANAGER_VERSION}" \
        -f "${MANIFESTS}/20-cert-manager/values.yaml" --wait
    kubectl apply -f "${MANIFESTS}/20-cert-manager/cluster-issuer-selfsigned.yaml"
}

apply_metallb() {
    has_layer metallb || return 0
    log "MetalLB install (${METALLB_VERSION})"
    kubectl apply -f "https://raw.githubusercontent.com/metallb/metallb/${METALLB_VERSION}/config/manifests/metallb-native.yaml"
    kubectl -n metallb-system wait --for=condition=Available deployment/controller --timeout=180s
    log "MetalLB IPAddressPool を動的生成（kind docker network 経由）"
    local pool_range
    pool_range="$(generate_metallb_pool)"
    log "  pool=${pool_range}"
    kubectl apply -f - <<EOF
apiVersion: metallb.io/v1beta1
kind: IPAddressPool
metadata:
  name: k1s0-local-pool
  namespace: metallb-system
spec:
  addresses:
    - ${pool_range}
  autoAssign: true
---
apiVersion: metallb.io/v1beta1
kind: L2Advertisement
metadata:
  name: k1s0-local-l2
  namespace: metallb-system
spec:
  ipAddressPools:
    - k1s0-local-pool
EOF
}

apply_istio() {
    has_layer istio || return 0
    log "Istio Ambient install"
    if ! command -v istioctl >/dev/null 2>&1; then
        warn "istioctl が PATH に無い。infra-ops / full プロファイルから実行してください"
        return 0
    fi
    istioctl install --set profile=ambient --skip-confirmation \
        -f "${MANIFESTS}/30-istio-ambient/values-ambient.yaml" || true
}

apply_kyverno() {
    has_layer kyverno || return 0
    log "Kyverno install (${KYVERNO_VERSION})"
    ensure_helm_repo kyverno https://kyverno.github.io/kyverno/
    helm upgrade --install kyverno kyverno/kyverno \
        --namespace kyverno --version "${KYVERNO_VERSION}" \
        -f "${MANIFESTS}/35-kyverno/values.yaml" --wait

    # baseline + drift policy を適用（ADR-POL-001 二分所有 + ADR-POL-002 SoT）
    log "  Kyverno ClusterPolicy (baseline + drift) を適用"
    kubectl apply -k "${REPO_ROOT}/infra/security/kyverno/" 2>&1 | tail -5 || warn "  policy apply 失敗"

    # ADR-POL-002 mode 切替: strict のみ drift policy を Enforce に patch
    if [[ "${MODE}" == "strict" ]]; then
        log "  mode=strict: drift policy を Enforce に切替"
        kubectl patch clusterpolicy block-non-canonical-helm-releases --type=json \
            -p='[{"op":"replace","path":"/spec/validationFailureAction","value":"Enforce"}]' \
            2>/dev/null || warn "  drift policy Enforce patch 失敗"
    else
        log "  mode=dev: drift policy は Audit のみ（違反は admission ではなく log で検知）"
    fi
}

apply_spire() {
    has_layer spire || return 0
    log "SPIRE install (umbrella ${SPIRE_VERSION} + crds ${SPIRE_CRDS_VERSION})"
    ensure_helm_repo spiffe https://spiffe.github.io/helm-charts-hardened/
    helm upgrade --install spire-crds spiffe/spire-crds \
        --namespace spire-system --version "${SPIRE_CRDS_VERSION}" --create-namespace
    helm upgrade --install spire spiffe/spire \
        --namespace spire-system --version "${SPIRE_VERSION}" \
        -f "${MANIFESTS}/40-spire/values.yaml" --wait || true
}

apply_dapr() {
    has_layer dapr || return 0
    log "Dapr install (${DAPR_VERSION})"
    ensure_helm_repo dapr https://dapr.github.io/helm-charts/
    helm upgrade --install dapr dapr/dapr \
        --namespace dapr-system --version "${DAPR_VERSION}" \
        -f "${MANIFESTS}/45-dapr/values.yaml" --wait
    log "Dapr Components を適用"
    if [[ -d "${REPO_ROOT}/tools/local-stack/dapr/components" ]]; then
        kubectl apply -f "${REPO_ROOT}/tools/local-stack/dapr/components/" || true
    fi
}

apply_flagd() {
    has_layer flagd || return 0
    log "flagd install"
    kubectl apply -f "${MANIFESTS}/50-flagd/manifest.yaml"
}

apply_gitea() {
    has_layer gitea || return 0
    log "Gitea install (Argo CD の sync 元 git)"
    kubectl apply -f "${REPO_ROOT}/infra/gitops/local-stack/gitea-deployment.yaml"
    kubectl apply -f "${REPO_ROOT}/infra/gitops/local-stack/gitea-service.yaml"
}

apply_registry() {
    has_layer registry || return 0
    log "OCI registry install (Kyverno ImageVerify 検証用)"
    kubectl apply -f "${REPO_ROOT}/infra/registry/local/deployment.yaml"
    kubectl apply -f "${REPO_ROOT}/infra/registry/local/service.yaml"
}

apply_argocd() {
    has_layer argocd || return 0
    log "Argo CD install (${ARGOCD_VERSION}, NodePort 30080)"
    ensure_helm_repo argo https://argoproj.github.io/argo-helm
    helm upgrade --install argocd argo/argo-cd \
        --namespace argocd --version "${ARGOCD_VERSION}" \
        -f "${MANIFESTS}/55-argocd/values.yaml" --wait || true

    # ADR-POL-002 (E 群解消): GitOps を SoT として確立する。
    # argocd 起動後、gitea に repo 内容を push し、deploy/apps/app-of-apps と
    # deploy/apps/application-sets の全 9 件を argocd に登録する（URL は local gitea に変換）。
    if has_layer gitea; then
        kubectl apply -f "${REPO_ROOT}/infra/gitops/local-stack/argocd-repo-secret.yaml" 2>/dev/null || true
        bootstrap_gitea_content
        apply_argocd_appsets
    fi
}

# gitea に local リポジトリ内容を push（idempotent）。
# argocd が repo を sync する前提として呼ばれる。
bootstrap_gitea_content() {
    log "  Gitea content bootstrap (admin user + argocd/k1s0 repo + push)"
    kubectl -n gitops wait --for=condition=Available deployment/gitea --timeout=300s 2>/dev/null \
        || { warn "  gitea が Available にならず bootstrap スキップ"; return; }
    local gitea_pod
    gitea_pod=$(kubectl -n gitops get pod -l app=gitea -o jsonpath='{.items[0].metadata.name}')

    # admin user 作成（既存なら skip）。gitea 1.22 系の admin user create CLI を使用。
    kubectl -n gitops exec "${gitea_pod}" -- /usr/local/bin/gitea admin user create \
        --username argocd --password 'ArgoCD123!' --email 'argocd@k1s0.local' --admin 2>/dev/null \
        || true

    # port-forward 経由で API + git push
    kubectl -n gitops port-forward svc/gitea 13000:3000 >/dev/null 2>&1 &
    local pf_pid=$!
    sleep 3

    # org / repo 作成（既存なら skip）
    curl -sf -u "argocd:ArgoCD123!" -X POST "http://localhost:13000/api/v1/orgs" \
        -H "Content-Type: application/json" \
        -d '{"username":"argocd","visibility":"public"}' >/dev/null 2>&1 || true
    curl -sf -u "argocd:ArgoCD123!" -X POST "http://localhost:13000/api/v1/orgs/argocd/repos" \
        -H "Content-Type: application/json" \
        -d '{"name":"k1s0","auto_init":false,"default_branch":"main","private":false}' >/dev/null 2>&1 || true

    # 現 working tree を push（commit 済み内容のみ）
    pushd "${REPO_ROOT}" >/dev/null
    git remote remove gitea-local 2>/dev/null || true
    git remote add gitea-local "http://argocd:ArgoCD123!@localhost:13000/argocd/k1s0.git"
    git push gitea-local HEAD:main --force 2>&1 | tail -3 || warn "  gitea push 失敗"
    git remote remove gitea-local 2>/dev/null || true
    popd >/dev/null

    kill ${pf_pid} 2>/dev/null || true
    wait ${pf_pid} 2>/dev/null || true
    log "  → gitea push 完了"
}

# deploy/apps/application-sets/*.yaml を local gitea URL に変換して直接適用。
# 設計判断 (ADR-POL-002):
#   - canonical 定義は production の GitHub URL のままで保持（deploy/apps/ 配下を改変しない）。
#   - local-stack ではコピー→sed 変換→apply の 3 段で local gitea URL に切替える。
#   - app-of-apps は production の GitHub root から再帰展開する pattern のため local-stack では使わない
#     （local で app-of-apps を入れると gitea 内の ApplicationSet が再び GitHub URL を読みに行き循環するため）。
apply_argocd_appsets() {
    log "  ApplicationSets を Argo CD に適用 (GitHub URL → local gitea URL 変換)"
    local tmp
    tmp=$(mktemp -d)
    cp "${REPO_ROOT}/deploy/apps/application-sets/"*.yaml "${tmp}/" 2>/dev/null || true
    find "${tmp}" -name "*.yaml" -exec sed -i \
        -e 's|https://github.com/k1s0/k1s0.git|http://gitea.gitops.svc.cluster.local:3000/argocd/k1s0.git|g' \
        -e 's|"https://github.com/k1s0/k1s0.git"|"http://gitea.gitops.svc.cluster.local:3000/argocd/k1s0.git"|g' \
        {} +
    kubectl apply -f "${tmp}/" 2>&1 | tail -10 || warn "  appset apply 失敗"
    rm -rf "${tmp}"
    local count
    count=$(ls "${REPO_ROOT}/deploy/apps/application-sets/"*.yaml 2>/dev/null | wc -l)
    log "  → ${count} ApplicationSets 適用完了"
}

apply_argo_rollouts() {
    has_layer argo-rollouts || return 0
    log "Argo Rollouts install (${ARGO_ROLLOUTS_VERSION}, ADR-CICD-002)"
    ensure_helm_repo argo https://argoproj.github.io/argo-helm
    helm upgrade --install argo-rollouts argo/argo-rollouts \
        --namespace argo-rollouts --version "${ARGO_ROLLOUTS_VERSION}" \
        -f "${MANIFESTS}/56-argo-rollouts/values.yaml" --wait || true
}

apply_envoy_gateway() {
    has_layer envoy-gateway || return 0
    log "Envoy Gateway install (${ENVOY_GATEWAY_VERSION}, OCI chart)"
    kubectl create namespace envoy-gateway-system 2>/dev/null || true
    helm upgrade --install envoy-gateway oci://docker.io/envoyproxy/gateway-helm \
        --namespace envoy-gateway-system --version "${ENVOY_GATEWAY_VERSION}" \
        -f "${MANIFESTS}/57-envoy-gateway/values.yaml" --wait || true
}

apply_cnpg() {
    has_layer cnpg || return 0
    log "CloudNativePG operator install (${CNPG_VERSION})"
    ensure_helm_repo cnpg https://cloudnative-pg.github.io/charts
    helm upgrade --install cnpg cnpg/cloudnative-pg \
        --namespace cnpg-system --version "${CNPG_VERSION}" \
        -f "${MANIFESTS}/60-cnpg/values.yaml" --wait
    log "k1s0 Postgres cluster を作成"
    kubectl apply -f "${MANIFESTS}/60-cnpg/k1s0-cluster.yaml"
}

apply_kafka() {
    has_layer kafka || return 0
    log "Strimzi Kafka operator install (${STRIMZI_VERSION})"
    ensure_helm_repo strimzi https://strimzi.io/charts/
    helm upgrade --install strimzi strimzi/strimzi-kafka-operator \
        --namespace kafka --version "${STRIMZI_VERSION}" \
        -f "${MANIFESTS}/65-kafka/strimzi-values.yaml" --wait
    log "k1s0 Kafka cluster (KRaft 単一ノード)"
    kubectl apply -f "${MANIFESTS}/65-kafka/k1s0-kafka.yaml"
}

apply_temporal() {
    has_layer temporal || return 0
    log "Temporal install (${TEMPORAL_VERSION}, workflow engine)"
    ensure_helm_repo temporal https://go.temporal.io/helm-charts
    helm upgrade --install temporal temporal/temporal \
        --namespace temporal --version "${TEMPORAL_VERSION}" \
        -f "${MANIFESTS}/66-temporal/values.yaml" --timeout 10m || true
}

apply_minio() {
    has_layer minio || return 0
    log "MinIO install"
    ensure_helm_repo minio https://charts.min.io/
    helm upgrade --install minio minio/minio \
        --namespace minio \
        -f "${MANIFESTS}/70-minio/values.yaml" --wait || true
}

apply_valkey() {
    has_layer valkey || return 0
    log "Valkey install (${VALKEY_VERSION})"
    ensure_helm_repo bitnami https://charts.bitnami.com/bitnami
    helm upgrade --install valkey bitnami/valkey \
        --namespace valkey --version "${VALKEY_VERSION}" \
        -f "${MANIFESTS}/75-valkey/values.yaml" --wait || true
}

apply_openbao() {
    has_layer openbao || return 0
    log "OpenBao (dev mode) install (${OPENBAO_VERSION})"
    ensure_helm_repo openbao https://openbao.github.io/openbao-helm
    helm upgrade --install openbao openbao/openbao \
        --namespace openbao --version "${OPENBAO_VERSION}" \
        -f "${MANIFESTS}/80-openbao/values.yaml" || true
}

apply_backstage() {
    has_layer backstage || return 0
    log "Backstage install (${BACKSTAGE_VERSION}, NodePort 30700)"
    ensure_helm_repo backstage https://backstage.github.io/charts
    helm upgrade --install backstage backstage/backstage \
        --namespace backstage --version "${BACKSTAGE_VERSION}" \
        -f "${MANIFESTS}/85-backstage/values.yaml" || true
}

apply_observability() {
    has_layer observability || return 0
    log "Observability (Loki ${LOKI_CHART_VERSION} / Tempo ${TEMPO_CHART_VERSION} / Grafana ${GRAFANA_CHART_VERSION} / OTel ${OTEL_COLLECTOR_VERSION}) install"
    ensure_helm_repo grafana https://grafana.github.io/helm-charts
    ensure_helm_repo open-telemetry https://open-telemetry.github.io/opentelemetry-helm-charts
    helm upgrade --install loki grafana/loki \
        --namespace observability --version "${LOKI_CHART_VERSION}" \
        -f "${MANIFESTS}/90-observability/values-loki.yaml" || true
    helm upgrade --install tempo grafana/tempo \
        --namespace observability --version "${TEMPO_CHART_VERSION}" \
        -f "${MANIFESTS}/90-observability/values-tempo.yaml" || true
    helm upgrade --install grafana grafana/grafana \
        --namespace observability --version "${GRAFANA_CHART_VERSION}" \
        -f "${MANIFESTS}/90-observability/values-grafana.yaml" || true
    # OTel Collector + Prometheus (現状 drift で見つかったため SoT に取り込む)
    ensure_helm_repo prometheus-community https://prometheus-community.github.io/helm-charts
    helm upgrade --install prometheus prometheus-community/prometheus \
        --namespace observability --version "${PROMETHEUS_CHART_VERSION}" \
        -f "${MANIFESTS}/90-observability/values-prometheus.yaml" || true
    helm upgrade --install otel-collector open-telemetry/opentelemetry-collector \
        --namespace observability --version "${OTEL_COLLECTOR_VERSION}" \
        -f "${MANIFESTS}/90-observability/values-otel-collector.yaml" || true
}

apply_keycloak() {
    has_layer keycloak || return 0
    log "Keycloak install (${KEYCLOAK_VERSION}, CNPG 連携)"
    ensure_helm_repo bitnami https://charts.bitnami.com/bitnami
    helm upgrade --install keycloak bitnami/keycloak \
        --namespace keycloak --version "${KEYCLOAK_VERSION}" \
        -f "${MANIFESTS}/95-keycloak/values.yaml" || true
}

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
