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

NS_COUNT="$(grep -c '^' "${NS_OUT}" || echo 0)"
RUNNING_PODS="$(grep -c 'Running' "${PODS_OUT}" || echo 0)"
TOTAL_PODS="$(($(wc -l < "${PODS_OUT}" | tr -d ' ') - 1))"
[[ "${TOTAL_PODS}" -lt 0 ]] && TOTAL_PODS=0

{
  echo "status: connected"
  echo "context: $(cat "${CTX_OUT}")"
  echo "namespace_count: ${NS_COUNT}"
  echo "running_pods: ${RUNNING_PODS}"
  echo "total_pods: ${TOTAL_PODS}"
} >> "${K8S_OUT}"

echo "=== k8s 軸 ==="
cat "${K8S_OUT}"
