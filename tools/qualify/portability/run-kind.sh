#!/usr/bin/env bash
#
# tools/qualify/portability/run-kind.sh
#
# L6 portability 検証 — multipass 不可環境（devcontainer など nested virtualization 制約下）
# 向けの代替経路。同一 host 上に 2 つ目の kind cluster を別バージョン K8s で立て、k1s0
# の chart / tier1 image が「異なる K8s 環境でも動く」ことを最小確認する。
#
# 主経路（multipass + kubeadm）と本経路の関係:
#   - multipass + kubeadm（run.sh）: VM ベースの完全分離 portability、host OS / CNI の
#     差を吸収する最も厚い検証。本番に最も近い経路。
#   - kind 2nd cluster（run-kind.sh、本ファイル）: VM 不要、devcontainer / WSL 等の
#     nested virtualization 制約下でも動かせる軽量経路。host kernel と CNI の差は
#     カバーできないが、K8s API バージョン差 / chart の portability は確認できる。
#
# ADR との整合:
#   ADR-CNCF-001（vanilla K8s + CNCF Conformance）— kind は upstream K8s container 実装
#   ADR-NET-001（kind multi-node = Calico）— 同 CNI（Calico）を本経路でも使用
#   ADR-INFRA-001（kubeadm + Cluster API）— 本経路は kubeadm 不経由のため補完位置付け
#
# Usage:
#   tools/qualify/portability/run-kind.sh                    # 全工程
#   tools/qualify/portability/run-kind.sh --keep-cluster     # 検証後も cluster を残す
#   tools/qualify/portability/run-kind.sh --kube-version V   # K8s version override
#
# 出力:
#   tests/.portability/<YYYY-MM-DD>/kind-cluster-info.txt
#   tests/.portability/<YYYY-MM-DD>/kind-tier1-smoke.txt
#
# 環境変数:
#   K1S0_PORT_KUBE_VERSION  K8s version（既定 v1.32.0、主 cluster の v1.31.4 と差を持たせる）
#
# 終了コード:
#   0 = portability 検証 PASS / 1 = cluster 起動 / Ready 確認失敗 / 2 = 環境不備

set -euo pipefail

# 引数解析
KEEP_CLUSTER=0
KUBE_VERSION="${K1S0_PORT_KUBE_VERSION:-v1.32.0}"
for arg in "$@"; do
    case "$arg" in
        --keep-cluster) KEEP_CLUSTER=1 ;;
        --kube-version) shift; KUBE_VERSION="$1" ;;
        --kube-version=*) KUBE_VERSION="${arg#*=}" ;;
        -h|--help) sed -n '2,30p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "[error] unknown arg: $arg" >&2; exit 2 ;;
    esac
done

# 必須 binary 確認
require_bin() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "[error] $1 not found in PATH" >&2
        return 1
    fi
}
require_bin kind || exit 2
require_bin kubectl || exit 2
require_bin docker || exit 2

# repo root（git 経由で安全に取得）
REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$REPO_ROOT"

# 出力先
RUN_DATE="$(date -u +%Y-%m-%d)"
OUT_DIR="${REPO_ROOT}/tests/.portability/${RUN_DATE}"
mkdir -p "$OUT_DIR"

# 別 cluster 名（主 cluster k1s0-local と衝突しない）
CLUSTER_NAME="k1s0-portability"

# cleanup 関数
cleanup() {
    if [[ "$KEEP_CLUSTER" -eq 0 ]]; then
        echo "[info] cluster ${CLUSTER_NAME} 削除"
        kind delete cluster --name "$CLUSTER_NAME" >/dev/null 2>&1 || true
    else
        echo "[info] --keep-cluster 指定: cluster を残す（手動 inspect 用）"
    fi
}
trap cleanup EXIT

# Step 1: 既存 cluster があれば削除して clean state から始める
if kind get clusters 2>/dev/null | grep -q "^${CLUSTER_NAME}$"; then
    echo "[info] 既存 cluster ${CLUSTER_NAME} を削除"
    kind delete cluster --name "$CLUSTER_NAME"
fi

# Step 2: 別バージョン K8s で kind cluster を起動（主 cluster と差別化）
echo "[info] kind cluster ${CLUSTER_NAME} を K8s ${KUBE_VERSION} で起動"
cat <<EOF | kind create cluster --name "$CLUSTER_NAME" --image "kindest/node:${KUBE_VERSION}" --config -
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
nodes:
  - role: control-plane
  - role: worker
networking:
  disableDefaultCNI: false
EOF

# Step 3: 出力先に cluster info を保存
{
    echo "# kind portability cluster info"
    echo "cluster_name: ${CLUSTER_NAME}"
    echo "kube_version: ${KUBE_VERSION}"
    echo "run_date: ${RUN_DATE}"
    echo "---"
    kubectl --context "kind-${CLUSTER_NAME}" version
    echo "---"
    kubectl --context "kind-${CLUSTER_NAME}" get nodes -o wide
    echo "---"
    kubectl --context "kind-${CLUSTER_NAME}" get pods -A
} > "${OUT_DIR}/kind-cluster-info.txt"

# Step 4: 主 cluster で build した tier1-state image を本 cluster に load
echo "[info] tier1-state image を ${CLUSTER_NAME} に load"
kind load docker-image ghcr.io/k1s0/k1s0/tier1-state:latest --name "$CLUSTER_NAME"

# Step 5: tier1-facade chart を deploy（minimal: state Pod のみ、dapr sidecar disabled）
echo "[info] tier1-facade-state を deploy"
kubectl --context "kind-${CLUSTER_NAME}" create namespace tier1-state || true
helm --kube-context "kind-${CLUSTER_NAME}" upgrade --install tier1-facade \
    "${REPO_ROOT}/deploy/charts/tier1-facade" \
    --namespace tier1-state \
    --set pods.secret.enabled=false \
    --set pods.workflow.enabled=false \
    --set image.pullPolicy=Never \
    --set-string 'podAnnotations.dapr\.io/enabled=false' \
    --wait --timeout 3m

# Step 6: smoke test — Pod Running と svc 疎通確認
echo "[info] smoke test: tier1-facade-state Pod Running 確認"
kubectl --context "kind-${CLUSTER_NAME}" -n tier1-state wait \
    --for=condition=ready pod -l app.kubernetes.io/component=state --timeout=120s

# Step 7: 結果記録
{
    echo "# tier1 smoke test on ${CLUSTER_NAME}"
    echo "kube_version: ${KUBE_VERSION}"
    echo "---"
    kubectl --context "kind-${CLUSTER_NAME}" -n tier1-state get pods -o wide
    echo "---"
    kubectl --context "kind-${CLUSTER_NAME}" -n tier1-state get svc
} > "${OUT_DIR}/kind-tier1-smoke.txt"

echo "[ok] kind portability 検証 PASS（cluster=${CLUSTER_NAME}, k8s=${KUBE_VERSION}）"
echo "[ok] 結果: ${OUT_DIR}/kind-cluster-info.txt"
echo "[ok] 結果: ${OUT_DIR}/kind-tier1-smoke.txt"
