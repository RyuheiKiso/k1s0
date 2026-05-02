#!/usr/bin/env bash
# C 軸: k8s 実機動作スナップショット
#
# 判定基準: docs/00_format/audit_criteria.md §C 軸
# 出力: ${EVIDENCE_DIR}/k8s-pods.txt
#       ${EVIDENCE_DIR}/k8s-namespaces.txt
#       ${EVIDENCE_DIR}/k8s-context.txt
#
# 設計原則:
#   - cluster が起動していなくても fail しない
#   - kubectl 不在 / cluster 接続失敗を「保留」として明示記録
#   - 嘘 PASS を書かないため、cluster なしの場合はその旨を出力に明記

set -euo pipefail
REPO_ROOT="$1"
EVIDENCE_DIR="$2"

K8S_OUT="${EVIDENCE_DIR}/k8s-snapshot.txt"
PODS_OUT="${EVIDENCE_DIR}/k8s-pods.txt"
NS_OUT="${EVIDENCE_DIR}/k8s-namespaces.txt"
CTX_OUT="${EVIDENCE_DIR}/k8s-context.txt"

{
  echo "# k8s 軸 スナップショット (生成: $(date -Iseconds))"
} > "${K8S_OUT}"

# kubectl 不在
if ! command -v kubectl >/dev/null 2>&1; then
  {
    echo "status: kubectl_not_installed"
    echo "note: 実機検証は保留。kubectl 未インストール環境では cluster 状態を取得できない"
  } >> "${K8S_OUT}"
  echo "=== k8s 軸: kubectl 不在のため保留 ==="
  cat "${K8S_OUT}"
  exit 0
fi

# cluster 接続不可
if ! kubectl cluster-info >/dev/null 2>&1; then
  {
    echo "status: cluster_unreachable"
    echo "note: 実機検証は保留。kubectl は存在するが cluster に接続不可"
    echo "last_known_check: SHIP_STATUS.md 参照（最終 commit hash で固定された証跡）"
  } >> "${K8S_OUT}"
  echo "=== k8s 軸: cluster 接続不可のため保留 ==="
  cat "${K8S_OUT}"
  exit 0
fi

# cluster 接続可: スナップショット取得
kubectl config current-context > "${CTX_OUT}" 2>&1 || echo "unknown" > "${CTX_OUT}"
kubectl get namespaces -o wide > "${NS_OUT}" 2>&1 || echo "FAILED" > "${NS_OUT}"
kubectl get pods --all-namespaces -o wide > "${PODS_OUT}" 2>&1 || echo "FAILED" > "${PODS_OUT}"

CTX_NAME="$(cat "${CTX_OUT}")"
# cluster 種別判定: kind / minikube / k3d / docker-desktop は production-equivalent ではない
# managed K8s（gke / eks / aks / oke / arn 等）は production-equivalent
# それ以外は unknown（嘘 PASS を防ぐため判定保留）
CLUSTER_CLASS="unknown"
case "${CTX_NAME}" in
  kind-*|*kind*)            CLUSTER_CLASS="kind" ;;
  minikube|*minikube*)      CLUSTER_CLASS="minikube" ;;
  k3d-*|*k3d*)              CLUSTER_CLASS="k3d" ;;
  docker-desktop|*docker-desktop*) CLUSTER_CLASS="docker-desktop" ;;
  gke_*|*gke*)              CLUSTER_CLASS="production-equivalent (GKE)" ;;
  arn:aws:eks:*|*eks*)      CLUSTER_CLASS="production-equivalent (EKS)" ;;
  *aks*)                    CLUSTER_CLASS="production-equivalent (AKS)" ;;
  *)                        CLUSTER_CLASS="unknown (要 context 確認)" ;;
esac

# kind / minikube / k3d / docker-desktop は local 段階。production-equivalent verification は別途必要
LOCAL_CLASSES_RE='^(kind|minikube|k3d|docker-desktop)$'
if [[ "${CLUSTER_CLASS}" =~ ${LOCAL_CLASSES_RE} ]]; then
  VERIFICATION_TIER="local-only (kind / minikube / k3d 等)"
  PRODUCTION_VERIFIED="false"
elif [[ "${CLUSTER_CLASS}" == production-equivalent* ]]; then
  VERIFICATION_TIER="production-equivalent (managed K8s)"
  PRODUCTION_VERIFIED="true"
else
  VERIFICATION_TIER="unknown"
  PRODUCTION_VERIFIED="false"
fi

NS_COUNT="$(grep -c '^' "${NS_OUT}" || echo 0)"
RUNNING_PODS="$(grep -c 'Running' "${PODS_OUT}" || echo 0)"
TOTAL_PODS="$(($(wc -l < "${PODS_OUT}" | tr -d ' ') - 1))"
[[ "${TOTAL_PODS}" -lt 0 ]] && TOTAL_PODS=0

{
  echo "status: connected"
  echo "context: ${CTX_NAME}"
  echo "cluster_class: ${CLUSTER_CLASS}"
  echo "verification_tier: ${VERIFICATION_TIER}"
  echo "production_verified: ${PRODUCTION_VERIFIED}"
  echo "namespace_count: ${NS_COUNT}"
  echo "running_pods: ${RUNNING_PODS}"
  echo "total_pods: ${TOTAL_PODS}"
} >> "${K8S_OUT}"

# 注意書き: local 段階の数値を production の証跡として誇張しない
if [[ "${PRODUCTION_VERIFIED}" == "false" ]]; then
  {
    echo
    echo "# 重要: 本 snapshot は local cluster (${CLUSTER_CLASS}) のみ。"
    echo "# production-equivalent (managed K8s) での E2E 検証は別途必要。"
    echo "# AUDIT.md の C 軸では local / production の 2 列で記録すること。"
  } >> "${K8S_OUT}"
fi

echo "=== k8s 軸 ==="
cat "${K8S_OUT}"
