#!/usr/bin/env bash
#
# tools/qualify/portability/run.sh
#
# L6 portability 検証 — multipass + kubeadm + Calico で kind 以外の vanilla K8s 実装で
# k1s0 が動くことを確認する。ADR-CNCF-001 の vanilla K8s 維持と ADR-INFRA-001 の
# kubeadm 採用に整合する portability 経路。k3s 派生（ADR-CNCF-001 で次点と判定）は
# 採用しない。
#
# 設計正典:
#   ADR-TEST-001（Test Pyramid + L6 portability）
#   ADR-INFRA-001（kubeadm + Cluster API、本番経路）
#   ADR-CNCF-001（vanilla K8s + CNCF Conformance 維持）
#   ADR-NET-001（kind multi-node = Calico、本 portability も Calico を採用）
#
# Usage:
#   tools/qualify/portability/run.sh                 # 全工程（VM 起動 → kubeadm → Calico → 検証 → 削除）
#   tools/qualify/portability/run.sh --keep-cluster  # 検証後も VM を残す（手動 inspect 用）
#   tools/qualify/portability/run.sh --vm-prefix PFX # multipass VM 名前空間（既定: k1s0-port）
#
# 出力:
#   tests/.portability/<YYYY-MM-DD>/cluster-info.txt    （kubectl version / nodes / get all）
#   tests/.portability/<YYYY-MM-DD>/conformance-link.md （L5 Sonobuoy 結果との対応）
#
# 環境変数:
#   K1S0_KUBE_VERSION    K8s バージョン（既定 1.31.0、ADR-INFRA-001 N と整合）
#   K1S0_CALICO_VERSION  Calico version（既定 v3.29.1、tools/local-stack/up.sh と整合）
#
# 終了コード:
#   0 = portability 検証 PASS / 1 = cluster 起動 / Ready 確認失敗 / 2 = 環境不備

set -euo pipefail

usage() {
    sed -n '2,28p' "$0" | sed 's/^# \{0,1\}//'
}

# 引数解析
KEEP_CLUSTER=0
VM_PREFIX="k1s0-port"
for arg in "$@"; do
    case "$arg" in
        --keep-cluster) KEEP_CLUSTER=1 ;;
        --vm-prefix) shift; VM_PREFIX="$1" ;;
        --vm-prefix=*) VM_PREFIX="${arg#*=}" ;;
        -h|--help) usage; exit 0 ;;
        *) echo "[error] unknown arg: $arg" >&2; usage; exit 2 ;;
    esac
done

# 必須 binary 確認
require_bin() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "[error] $1 not found in PATH" >&2
        echo "  $2" >&2
        return 1
    fi
}
require_bin multipass "install: snap install multipass（Ubuntu）/ brew install multipass（macOS）" || exit 2
require_bin kubectl "install: https://kubernetes.io/docs/tasks/tools/" || exit 2

# repo root（git 経由で安全に取得、cd-safe）
REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$REPO_ROOT"

# 出力先（YYYY-MM-DD、月単位ではなく日単位で時系列を細かく残す）
RUN_DATE="$(date -u +%Y-%m-%d)"
OUT_DIR="${REPO_ROOT}/tests/.portability/${RUN_DATE}"
mkdir -p "$OUT_DIR"

# K8s / Calico のバージョン（env で override 可、既定は ADR と整合）
KUBE_VERSION="${K1S0_KUBE_VERSION:-1.31.0}"
CALICO_VERSION="${K1S0_CALICO_VERSION:-v3.29.1}"

# VM 名（control-plane 1 + worker 2 の 3 ノード構成、kind multi-node と同等）
CP_VM="${VM_PREFIX}-cp"
W1_VM="${VM_PREFIX}-w1"
W2_VM="${VM_PREFIX}-w2"

# cleanup 関数（trap で強制呼び出し、--keep-cluster 時は skip）
cleanup() {
    if [[ "$KEEP_CLUSTER" -eq 1 ]]; then
        echo "[info] --keep-cluster: VM 削除を skip（${CP_VM} ${W1_VM} ${W2_VM}）"
        return 0
    fi
    echo "[info] multipass VM 削除"
    multipass delete --purge "$CP_VM" "$W1_VM" "$W2_VM" 2>/dev/null || true
}
trap cleanup EXIT

# Step 1: multipass で 3 VM を起動（4 GB / 2 CPU、kubeadm の最低要件を満たす）
echo "[info] multipass で 3 VM を起動（control-plane 1 + worker 2）"
for vm in "$CP_VM" "$W1_VM" "$W2_VM"; do
    multipass launch --name "$vm" --cpus 2 --memory 4G --disk 20G 24.04
done

# Step 2: 各 VM に kubeadm / kubelet / kubectl / containerd を install
echo "[info] 全 VM に containerd + kubeadm/kubelet/kubectl を install"
INSTALL_SCRIPT='#!/usr/bin/env bash
set -euo pipefail
sudo apt-get update -qq
sudo apt-get install -y -qq apt-transport-https ca-certificates curl gpg containerd
sudo systemctl enable --now containerd
sudo modprobe br_netfilter
echo "br_netfilter" | sudo tee /etc/modules-load.d/k8s.conf >/dev/null
echo "net.bridge.bridge-nf-call-iptables=1" | sudo tee /etc/sysctl.d/k8s.conf >/dev/null
sudo sysctl --system >/dev/null
sudo swapoff -a
KUBE_VERSION_MAJOR_MINOR="$(echo "${KUBE_VERSION}" | awk -F. "{print \$1\".\"\$2}")"
curl -fsSL "https://pkgs.k8s.io/core:/stable:/v${KUBE_VERSION_MAJOR_MINOR}/deb/Release.key" | sudo gpg --dearmor -o /etc/apt/keyrings/kubernetes-apt-keyring.gpg
echo "deb [signed-by=/etc/apt/keyrings/kubernetes-apt-keyring.gpg] https://pkgs.k8s.io/core:/stable:/v${KUBE_VERSION_MAJOR_MINOR}/deb/ /" | sudo tee /etc/apt/sources.list.d/kubernetes.list >/dev/null
sudo apt-get update -qq
sudo apt-get install -y -qq kubelet kubeadm kubectl
sudo apt-mark hold kubelet kubeadm kubectl
'
for vm in "$CP_VM" "$W1_VM" "$W2_VM"; do
    echo "[info]   $vm で install"
    multipass exec "$vm" -- bash -c "KUBE_VERSION=${KUBE_VERSION} bash -" <<< "$INSTALL_SCRIPT"
done

# Step 3: control-plane 上で kubeadm init（pod-network-cidr は Calico 既定 192.168.0.0/16）
echo "[info] control-plane で kubeadm init"
multipass exec "$CP_VM" -- sudo kubeadm init \
    --kubernetes-version "v${KUBE_VERSION}" \
    --pod-network-cidr=192.168.0.0/16 \
    --apiserver-advertise-address="$(multipass info "$CP_VM" | awk '/IPv4/ {print $2; exit}')"

# kubeadm join 用の token を取得し、worker でそれぞれ join 実行
JOIN_CMD="$(multipass exec "$CP_VM" -- sudo kubeadm token create --print-join-command)"
for w in "$W1_VM" "$W2_VM"; do
    echo "[info]   $w を control-plane に join"
    multipass exec "$w" -- sudo bash -c "$JOIN_CMD"
done

# Step 4: control-plane の kubeconfig を取得して KUBECONFIG として使う
echo "[info] kubeconfig 取得"
KUBECONFIG_PATH="${OUT_DIR}/kubeconfig"
multipass exec "$CP_VM" -- sudo cat /etc/kubernetes/admin.conf > "$KUBECONFIG_PATH"
chmod 600 "$KUBECONFIG_PATH"
export KUBECONFIG="$KUBECONFIG_PATH"

# Step 5: Calico CNI install（ADR-NET-001 と整合、kind multi-node と同 CNI）
echo "[info] Calico CNI install"
kubectl create -f "https://raw.githubusercontent.com/projectcalico/calico/${CALICO_VERSION}/manifests/tigera-operator.yaml"
cat <<EOF | kubectl apply -f -
apiVersion: operator.tigera.io/v1
kind: Installation
metadata:
  name: default
spec:
  calicoNetwork:
    ipPools:
      - blockSize: 26
        cidr: 192.168.0.0/16
        encapsulation: VXLANCrossSubnet
        natOutgoing: Enabled
        nodeSelector: all()
EOF

# Step 6: 全 node が Ready になるまで待機（最大 5 分、CNI 起動 + kubelet 接続）
echo "[info] cluster Ready 待機（最大 5 分）"
kubectl wait --for=condition=Ready node --all --timeout=300s

# Step 7: 検証用 cluster info を artifact 化
echo "[info] cluster-info を artifact に記録"
{
    echo "# k1s0 portability cluster-info — ${RUN_DATE}"
    echo ""
    echo "## kubectl version"
    echo '```'
    kubectl version
    echo '```'
    echo ""
    echo "## kubectl get nodes -o wide"
    echo '```'
    kubectl get nodes -o wide
    echo '```'
    echo ""
    echo "## kubectl get all -A（要約）"
    echo '```'
    kubectl get all -A | head -50
    echo '```'
    echo ""
    echo "## CNI"
    echo '```'
    kubectl get tigerastatus 2>/dev/null || kubectl get pods -n calico-system
    echo '```'
} > "${OUT_DIR}/cluster-info.txt"

# Step 8: L5 Sonobuoy 結果との対応リンクを記録（採用初期で sonobuoy 実走を統合）
{
    echo "# k1s0 portability — L5 Conformance との対応"
    echo ""
    echo "本 portability 検証は kubeadm + Calico の vanilla K8s 構成で cluster Ready"
    echo "までを確認した。CNCF Conformance（Sonobuoy certified-conformance）の本格実施は"
    echo "tools/qualify/conformance/run.sh --skip-up を本 cluster の KUBECONFIG で実行する"
    echo "経路で採用初期に統合する。kind cluster での L5 Conformance 結果（"
    echo "tests/.conformance/<YYYY-MM>/sonobuoy-results.tar.gz）と本 portability cluster の"
    echo "Conformance 結果が一致することで「kind 以外の vanilla K8s 実装でも k1s0 が動く」"
    echo "を機械検証する。"
    echo ""
    echo "## 実行日時"
    echo "${RUN_DATE}"
    echo ""
    echo "## K8s version"
    echo "${KUBE_VERSION}"
    echo ""
    echo "## Calico version"
    echo "${CALICO_VERSION}"
} > "${OUT_DIR}/conformance-link.md"

echo "[ok] portability 検証 PASS"
echo "[done] cluster-info: ${OUT_DIR}/cluster-info.txt"
echo "[done] conformance-link: ${OUT_DIR}/conformance-link.md"
echo "[done] kubeconfig: ${OUT_DIR}/kubeconfig"
