#!/usr/bin/env bash
#
# tools/e2e/owner/up.sh — owner suite cluster の起動（multipass × 5 + kubeadm 3CP HA）
#
# 設計正典:
#   ADR-TEST-008（owner / user 二分構造、owner = 48GB host 専用）
#   ADR-INFRA-001（kubeadm + Cluster API、本番経路）
#   ADR-NET-001（production CNI = Cilium）
#   ADR-STOR-001（CSI = Longhorn）
#   ADR-STOR-002（LB = MetalLB）
#   ADR-POL-002（local-stack を構成 SoT に統一: install/manifests を再利用）
#   docs/05_実装/30_CI_CD設計/35_e2e_test_design/10_owner_suite/01_環境契約.md
#
# 前提環境:
#   - host OS の WSL2 native shell（devcontainer 内では multipass 不可、nested virt 制約）
#   - 48GB host RAM（multipass × 5 で約 30GB + フルスタック 12GB + host 6GB）
#   - multipass / kubectl / helm / kind が install 済（不在時は案内表示）
#
# Usage:
#   tools/e2e/owner/up.sh                       # フル起動（VM + kubeadm + Cilium + Longhorn + MetalLB + フルスタック）
#   tools/e2e/owner/up.sh --skip-stack          # cluster + CNI/CSI/LB のみ、フルスタック非適用（debug 用）
#   tools/e2e/owner/up.sh --keep-cluster        # 起動のみ（trap cleanup 無効化、down.sh で別途削除）
#   tools/e2e/owner/up.sh --vm-prefix PFX       # multipass VM 名前空間（既定: k1s0-owner）
#
# 環境変数:
#   K1S0_KUBE_VERSION       K8s バージョン（既定 1.31.0、ADR-INFRA-001 N と整合）
#   K1S0_CILIUM_VERSION     Cilium chart version（既定 1.16.5）
#   K1S0_LONGHORN_VERSION   Longhorn chart version（既定 1.7.2）
#   K1S0_METALLB_VERSION    MetalLB version（既定 v0.14.9）
#
# 出力:
#   tests/.owner-e2e/<YYYY-MM-DD>/kubeconfig         （kubeconfig path、context = k1s0-owner-e2e）
#   tests/.owner-e2e/<YYYY-MM-DD>/cluster-info.txt    （起動完了時の cluster 状態）
#
# 終了コード:
#   0 = cluster 起動完了 / 1 = cluster bootstrap 失敗 / 2 = 引数 / 環境不備

set -euo pipefail

# 自スクリプトのディレクトリを取得（lib/ への相対パス解決用）
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIB_DIR="$(cd "${SCRIPT_DIR}/../lib" && pwd)"

# 共通 helper を順次 source
# shellcheck source=../lib/common.sh
source "${LIB_DIR}/common.sh"
# shellcheck source=../lib/multipass.sh
source "${LIB_DIR}/multipass.sh"
# shellcheck source=../lib/kubeadm.sh
source "${LIB_DIR}/kubeadm.sh"
# shellcheck source=../lib/cluster_components.sh
source "${LIB_DIR}/cluster_components.sh"
# shellcheck source=../lib/artifact.sh
source "${LIB_DIR}/artifact.sh"

# 引数解析の既定値
SKIP_STACK=0
KEEP_CLUSTER=0
VM_PREFIX="k1s0-owner"

# Usage 出力（ヘッダコメントの 9〜30 行目を抜粋）
usage() {
    sed -n '9,30p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
}

# 引数解析（位置引数なし、フラグのみ）
while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help) usage; exit 0 ;;
        --skip-stack) SKIP_STACK=1; shift ;;
        --keep-cluster) KEEP_CLUSTER=1; shift ;;
        --vm-prefix) VM_PREFIX="$2"; shift 2 ;;
        --vm-prefix=*) VM_PREFIX="${1#*=}"; shift ;;
        *) e2e_warn "未知の引数: $1"; usage; exit 2 ;;
    esac
done

# 必須 binary 確認（不在時は fatal）
e2e_require_bin multipass "snap install multipass（Ubuntu）/ brew install multipass（macOS）" \
    || e2e_fail "multipass がない（owner suite は host OS の WSL2 native shell から実行する前提）"
e2e_require_bin kubectl "https://kubernetes.io/docs/tasks/tools/" \
    || e2e_fail "kubectl がない"
e2e_require_bin helm "https://helm.sh/docs/intro/install/" \
    || e2e_fail "helm がない（Cilium / Longhorn install で使う）"

# repo root + 実走日 + artifact 出力先を確定
REPO_ROOT="$(e2e_repo_root)"
RUN_DATE="$(e2e_run_date)"
ARTIFACT_DIR="$(e2e_owner_artifact_dir "${RUN_DATE}")"
mkdir -p "${ARTIFACT_DIR}"

# バージョン（env で override 可、既定は本番設計と整合）
KUBE_VERSION="${K1S0_KUBE_VERSION:-1.31.0}"
CILIUM_VERSION="${K1S0_CILIUM_VERSION:-1.16.5}"
LONGHORN_VERSION="${K1S0_LONGHORN_VERSION:-1.7.2}"
METALLB_VERSION="${K1S0_METALLB_VERSION:-v0.14.9}"

# VM 名（control-plane 1/2/3 + worker 1/2 の 5 ノード構成、ADR-INFRA-001 3CP HA）
CP1_VM="${VM_PREFIX}-cp-1"
CP2_VM="${VM_PREFIX}-cp-2"
CP3_VM="${VM_PREFIX}-cp-3"
W1_VM="${VM_PREFIX}-w-1"
W2_VM="${VM_PREFIX}-w-2"
ALL_VMS=("${CP1_VM}" "${CP2_VM}" "${CP3_VM}" "${W1_VM}" "${W2_VM}")

# cleanup 関数（trap で強制呼び出し、--keep-cluster 時は skip）
cleanup() {
    if [[ "${KEEP_CLUSTER}" -eq 1 ]]; then
        e2e_log "--keep-cluster: VM 削除を skip（${ALL_VMS[*]}）"
        return 0
    fi
    if [[ "${E2E_OWNER_UP_SUCCESS:-0}" -eq 1 ]]; then
        # 正常完了時は VM を残す（後続 test 実行のため）。明示削除は down.sh で。
        e2e_log "起動完了、VM は維持（cleanup は down.sh で実行）"
        return 0
    fi
    e2e_warn "起動失敗 / 中断、cleanup として VM 削除"
    e2e_multipass_delete_all "${ALL_VMS[@]}"
}
trap cleanup EXIT

# Step 1: multipass で 5 VM を起動（control-plane 3 + worker 2、各 6GB / 2vCPU / 20GB）
e2e_log "[Step 1/8] multipass × 5 起動（3 CP HA + 2 W）"
for vm in "${ALL_VMS[@]}"; do
    e2e_multipass_launch "${vm}" 2 6G 20G "24.04"
done

# Step 2: 全 VM に containerd + kubeadm/kubelet/kubectl install
e2e_log "[Step 2/8] 全 VM に containerd + kubeadm/kubelet/kubectl (v${KUBE_VERSION}) install"
for vm in "${ALL_VMS[@]}"; do
    e2e_multipass_install_k8s "${vm}" "${KUBE_VERSION}"
done

# Longhorn の前提として open-iscsi を全 VM に install（kubeadm 直後に実施）
e2e_log "[Step 2.5/8] Longhorn 前提: open-iscsi install"
for vm in "${ALL_VMS[@]}"; do
    e2e_multipass_exec "${vm}" sudo apt-get install -y -qq open-iscsi
    e2e_multipass_exec "${vm}" sudo systemctl enable --now iscsid
done

# Step 3: control-plane 1 で kubeadm init + cert upload
e2e_log "[Step 3/8] control-plane 1 で kubeadm init + cert upload"
JOIN_OUTPUT="$(e2e_kubeadm_init_cp1 "${CP1_VM}" "${KUBE_VERSION}")"
JOIN_CP_CMD="$(echo "${JOIN_OUTPUT}" | grep '^JOIN_CP_CMD=' | sed 's/^JOIN_CP_CMD=//')"
JOIN_W_CMD="$(echo "${JOIN_OUTPUT}" | grep '^JOIN_W_CMD=' | sed 's/^JOIN_W_CMD=//')"
if [[ -z "${JOIN_CP_CMD}" || -z "${JOIN_W_CMD}" ]]; then
    e2e_fail "kubeadm join コマンド取得失敗"
fi

# Step 4: 追加 control-plane を join（HA 構成）
e2e_log "[Step 4/8] cp-2 / cp-3 を control-plane に join（HA）"
e2e_kubeadm_join_cp "${CP2_VM}" "${JOIN_CP_CMD}"
e2e_kubeadm_join_cp "${CP3_VM}" "${JOIN_CP_CMD}"

# Step 5: worker を join
e2e_log "[Step 5/8] w-1 / w-2 を worker として join"
e2e_kubeadm_join_worker "${W1_VM}" "${JOIN_W_CMD}"
e2e_kubeadm_join_worker "${W2_VM}" "${JOIN_W_CMD}"

# Step 6: kubeconfig 取得 + KUBECONFIG 環境変数を設定
KUBECONFIG_PATH="${ARTIFACT_DIR}/kubeconfig"
e2e_kubeadm_fetch_kubeconfig "${CP1_VM}" "${KUBECONFIG_PATH}"
export KUBECONFIG="${KUBECONFIG_PATH}"
e2e_log "[Step 6/8] kubeconfig: ${KUBECONFIG_PATH} (context = k1s0-owner-e2e)"

# Step 7: 疑似 multi-AZ topology label + Cilium + Longhorn + MetalLB install
e2e_log "[Step 7/8] CNI (Cilium) + CSI (Longhorn) + LB (MetalLB) install"
# control-plane Ready 前に node label を付けるため node 一覧待機
kubectl wait --for=jsonpath='{.status.conditions[?(@.type=="NodeStatusUpdate")].status}' \
    node --all --timeout=120s 2>/dev/null || true
# zone label は worker 2 台に付ける（control-plane は zone 不問、ワークロード対象外）
e2e_kubeadm_label_zone "${KUBECONFIG_PATH}" "${W1_VM}" "${W2_VM}"
# Cilium / Longhorn / MetalLB を順次 install
e2e_install_cilium "${CILIUM_VERSION}"
e2e_wait_nodes_ready 600
e2e_install_longhorn "${LONGHORN_VERSION}"
e2e_install_metallb "${METALLB_VERSION}"

# Step 8: フルスタック install（local-stack/manifests/ を再利用、--skip-stack で省略可）
if [[ "${SKIP_STACK}" -eq 0 ]]; then
    e2e_log "[Step 8/8] フルスタック install (local-stack/lib/apply-layers.sh 経由、cni/storageclass-kind-patch を skip)"
    # local-stack の up.sh を --no-cluster --skip cni で呼ぶ。
    # CNI (Cilium) / CSI (Longhorn) / LB (MetalLB) は本 owner/up.sh で先に install 済のため skip。
    # role=full で 11 components すべてを apply（ADR-POL-002 SoT 再利用）。
    "${REPO_ROOT}/tools/local-stack/up.sh" \
        --no-cluster \
        --role full \
        --skip cni \
        --mode strict \
        || e2e_warn "フルスタック install 中に warning（個別の retry は Runbook 参照）"
else
    e2e_log "[Step 8/8] --skip-stack: フルスタック install を skip（CNI/CSI/LB のみで終了）"
fi

# 起動完了 marker（cleanup trap が VM を残すように）
E2E_OWNER_UP_SUCCESS=1

# cluster-info を artifact 化（後続の cut.sh / Runbook で参照）
e2e_collect_cluster_info "${ARTIFACT_DIR}"

# 完了報告
e2e_log "owner suite cluster 起動完了"
e2e_log "  KUBECONFIG: ${KUBECONFIG_PATH}"
e2e_log "  context:    k1s0-owner-e2e"
e2e_log "  artifact:   ${ARTIFACT_DIR}"
e2e_log ""
e2e_log "次の操作:"
e2e_log "  KUBECONFIG=${KUBECONFIG_PATH} make e2e-owner-full       # 全 8 部位実行"
e2e_log "  KUBECONFIG=${KUBECONFIG_PATH} make e2e-owner-platform   # 部位個別実行"
e2e_log "  ./tools/e2e/owner/down.sh                               # cluster 削除"
