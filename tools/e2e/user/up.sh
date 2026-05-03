#!/usr/bin/env bash
#
# tools/e2e/user/up.sh — user suite cluster の起動（kind + minimum stack）
#
# 設計正典:
#   ADR-TEST-008（owner / user 二分構造、user = 16GB host OK）
#   ADR-NET-001（kind multi-node = Calico）
#   ADR-POL-002（local-stack の install/manifests を再利用）
#   docs/05_実装/30_CI_CD設計/35_e2e_test_design/20_user_suite/01_環境契約.md
#
# 前提環境:
#   - 16GB host RAM 推奨（minimum stack 約 1.7GB + kind 約 4GB + 利用者 dev 余裕 ≈ 10GB 必要）
#   - kind / kubectl / helm が install 済（不在時は案内表示）
#   - devcontainer 内でも起動可能（multipass 不要）
#
# Usage:
#   tools/e2e/user/up.sh                       # 既定 minimum stack（Dapr + tier1 facade + Keycloak + CNPG）
#   tools/e2e/user/up.sh --add workflow        # 任意 stack 追加（複数指定可、--add foo --add bar）
#   tools/e2e/user/up.sh --backend minio       # default backend を CNPG → MinIO に変更
#   tools/e2e/user/up.sh --keep-cluster        # 起動のみ（既存 cluster 再利用）
#
# 環境変数:
#   K1S0_USER_E2E_KEEP_CLUSTER=1   既存 kind cluster を再利用（同名 cluster 削除しない）
#
# 出力:
#   tests/.user-e2e/<YYYY-MM-DD>/cluster-info.txt
#
# 終了コード:
#   0 = cluster 起動完了 / 1 = cluster bootstrap 失敗 / 2 = 引数 / 環境不備

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIB_DIR="$(cd "${SCRIPT_DIR}/../lib" && pwd)"

# shellcheck source=../lib/common.sh
source "${LIB_DIR}/common.sh"
# shellcheck source=../lib/artifact.sh
source "${LIB_DIR}/artifact.sh"

# kind cluster 名（owner suite の context と衝突しない名前を使う）
KIND_CLUSTER_NAME="k1s0-user-e2e"
DEFAULT_BACKEND="cnpg"
ADD_COMPONENTS=()
KEEP_CLUSTER="${K1S0_USER_E2E_KEEP_CLUSTER:-0}"

usage() {
    sed -n '8,25p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
}

# 引数解析（--add は複数回指定可）
while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help) usage; exit 0 ;;
        --add) ADD_COMPONENTS+=("$2"); shift 2 ;;
        --add=*) ADD_COMPONENTS+=("${1#*=}"); shift ;;
        --backend) DEFAULT_BACKEND="$2"; shift 2 ;;
        --backend=*) DEFAULT_BACKEND="${1#*=}"; shift ;;
        --keep-cluster) KEEP_CLUSTER=1; shift ;;
        *) e2e_warn "未知の引数: $1"; usage; exit 2 ;;
    esac
done

# 必須 binary 確認
e2e_require_bin kind "https://kind.sigs.k8s.io/docs/user/quick-start/" \
    || e2e_fail "kind がない"
e2e_require_bin kubectl "https://kubernetes.io/docs/tasks/tools/" \
    || e2e_fail "kubectl がない"
e2e_require_bin helm "https://helm.sh/docs/intro/install/" \
    || e2e_fail "helm がない"

REPO_ROOT="$(e2e_repo_root)"
RUN_DATE="$(e2e_run_date)"
ARTIFACT_DIR="$(e2e_user_artifact_dir "${RUN_DATE}")"
mkdir -p "${ARTIFACT_DIR}"

# Step 1: kind cluster 起動（既存なら skip）
e2e_log "[Step 1/4] kind cluster 起動 (${KIND_CLUSTER_NAME})"
if kind get clusters 2>/dev/null | grep -q "^${KIND_CLUSTER_NAME}$"; then
    if [[ "${KEEP_CLUSTER}" -eq 1 ]]; then
        e2e_log "  既存 cluster を再利用 (--keep-cluster)"
    else
        e2e_log "  既存 cluster を削除して fresh 起動"
        kind delete cluster --name "${KIND_CLUSTER_NAME}"
    fi
fi
if ! kind get clusters 2>/dev/null | grep -q "^${KIND_CLUSTER_NAME}$"; then
    # control-plane 1 + worker 1 の最小構成（local-stack/kind-cluster.yaml の subset を inline 使用）
    kind create cluster --name "${KIND_CLUSTER_NAME}" --config - <<EOF
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
nodes:
  - role: control-plane
  - role: worker
EOF
fi
# kubeconfig context を current にする
kubectl config use-context "kind-${KIND_CLUSTER_NAME}"

# Step 2: minimum stack の install（local-stack/up.sh を --no-cluster で呼び、必要 layer のみ）
# minimum stack = cni (Calico) + cert-manager + dapr + cnpg/minio (backend) + keycloak
e2e_log "[Step 2/4] minimum stack install (cert-manager + dapr + ${DEFAULT_BACKEND} + keycloak)"
LAYERS="cni,cert-manager,dapr,${DEFAULT_BACKEND},keycloak"
"${REPO_ROOT}/tools/local-stack/up.sh" \
    --no-cluster \
    --layers "${LAYERS}" \
    --mode dev

# Step 3: 任意 stack の opt-in install（--add で指定された component を追加適用）
if [[ ${#ADD_COMPONENTS[@]} -gt 0 ]]; then
    e2e_log "[Step 3/4] 任意 stack 追加 install (${ADD_COMPONENTS[*]})"
    # 同時起動 2 個まで（ADR-TEST-008 の 16GB 制約）
    if [[ ${#ADD_COMPONENTS[@]} -gt 2 ]]; then
        e2e_warn "任意 stack 同時起動が 3 個以上です（${#ADD_COMPONENTS[@]} 個）。16GB host で OOM の risk"
    fi
    ADD_LAYERS="$(IFS=','; echo "${ADD_COMPONENTS[*]}")"
    "${REPO_ROOT}/tools/local-stack/up.sh" \
        --no-cluster \
        --layers "${ADD_LAYERS}" \
        --mode dev
else
    e2e_log "[Step 3/4] 任意 stack opt-in なし、minimum stack のみで終了"
fi

# Step 4: cluster-info を artifact 化
e2e_log "[Step 4/4] cluster-info 出力"
e2e_collect_cluster_info "${ARTIFACT_DIR}"

# 完了報告
e2e_log "user suite cluster 起動完了"
e2e_log "  context:  kind-${KIND_CLUSTER_NAME}"
e2e_log "  artifact: ${ARTIFACT_DIR}"
e2e_log ""
e2e_log "次の操作:"
e2e_log "  make e2e-user-smoke    # smoke test 実行（5 分以内）"
e2e_log "  make e2e-user-full     # 全 test 実行（30〜45 分）"
e2e_log "  ./tools/e2e/user/down.sh   # cluster 削除"
