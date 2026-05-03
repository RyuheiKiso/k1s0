#!/usr/bin/env bash
#
# tools/e2e/lib/multipass.sh — multipass VM 操作 helper（owner suite 専用）
#
# 設計正典:
#   ADR-TEST-008（owner suite 環境契約: multipass × 5 + kubeadm 3CP HA + 2W）
#   docs/05_実装/30_CI_CD設計/35_e2e_test_design/10_owner_suite/01_環境契約.md
#
# 直接実行は不可。owner/up.sh / down.sh から `source` で読み込む。
# 前提: tools/e2e/lib/common.sh が source 済（e2e_log / e2e_warn / e2e_fail を使う）
#
# 提供関数:
#   e2e_multipass_launch     — 1 VM 起動（既存なら skip）
#   e2e_multipass_install_k8s — VM 内に containerd + kubeadm/kubelet/kubectl install
#   e2e_multipass_exec        — VM 内で任意コマンド実行
#   e2e_multipass_ip          — VM の IPv4 取得
#   e2e_multipass_delete_all  — VM 一覧をまとめて削除
#   e2e_multipass_running     — VM が Running 状態か確認

if [[ -n "${E2E_LIB_MULTIPASS_LOADED:-}" ]]; then
    return 0
fi
readonly E2E_LIB_MULTIPASS_LOADED=1

# 1 VM を multipass で起動。既存なら skip（冪等性確保）
e2e_multipass_launch() {
    # 引数 1: VM 名 / 引数 2: CPU / 引数 3: メモリ（例 6G）/ 引数 4: disk（例 20G） / 引数 5: image（例 24.04）
    local vm_name="$1"
    local cpus="$2"
    local memory="$3"
    local disk="$4"
    local image="${5:-24.04}"

    # 既存 VM 確認（multipass info で exit 0 なら存在）
    if multipass info "${vm_name}" >/dev/null 2>&1; then
        e2e_log "  ${vm_name} は既存（再利用）"
        return 0
    fi

    e2e_log "  ${vm_name} 起動（cpu=${cpus} memory=${memory} disk=${disk} image=${image}）"
    multipass launch \
        --name "${vm_name}" \
        --cpus "${cpus}" \
        --memory "${memory}" \
        --disk "${disk}" \
        "${image}"
}

# VM 内に containerd + kubeadm/kubelet/kubectl を install
# K8s バージョンは引数で渡す（呼び出し側で K1S0_KUBE_VERSION 等から取得）
e2e_multipass_install_k8s() {
    # 引数 1: VM 名 / 引数 2: K8s バージョン（例 1.31.0）
    local vm_name="$1"
    local kube_version="$2"
    local kube_minor
    kube_minor="$(echo "${kube_version}" | awk -F. '{print $1"."$2}')"

    # K8s install script（VM 内で実行）。pkgs.k8s.io 公式 repo を採用、ADR-INFRA-001 と整合
    local install_script
    install_script=$(cat <<EOF
#!/usr/bin/env bash
set -euo pipefail
sudo apt-get update -qq
sudo apt-get install -y -qq apt-transport-https ca-certificates curl gpg containerd
sudo systemctl enable --now containerd
# kernel module + sysctl 設定（kubeadm の前提）
sudo modprobe br_netfilter
echo "br_netfilter" | sudo tee /etc/modules-load.d/k8s.conf >/dev/null
echo "net.bridge.bridge-nf-call-iptables=1" | sudo tee /etc/sysctl.d/k8s.conf >/dev/null
echo "net.ipv4.ip_forward=1" | sudo tee -a /etc/sysctl.d/k8s.conf >/dev/null
sudo sysctl --system >/dev/null
sudo swapoff -a
# kubernetes apt repo（v${kube_minor} 系）
sudo mkdir -p /etc/apt/keyrings
curl -fsSL "https://pkgs.k8s.io/core:/stable:/v${kube_minor}/deb/Release.key" | sudo gpg --dearmor -o /etc/apt/keyrings/kubernetes-apt-keyring.gpg
echo "deb [signed-by=/etc/apt/keyrings/kubernetes-apt-keyring.gpg] https://pkgs.k8s.io/core:/stable:/v${kube_minor}/deb/ /" | sudo tee /etc/apt/sources.list.d/kubernetes.list >/dev/null
sudo apt-get update -qq
sudo apt-get install -y -qq kubelet kubeadm kubectl
sudo apt-mark hold kubelet kubeadm kubectl
# containerd の SystemdCgroup を有効化（kubelet との整合）
sudo mkdir -p /etc/containerd
containerd config default | sudo tee /etc/containerd/config.toml >/dev/null
sudo sed -i 's/SystemdCgroup = false/SystemdCgroup = true/' /etc/containerd/config.toml
sudo systemctl restart containerd
EOF
)

    e2e_log "  ${vm_name} に containerd + kubeadm/kubelet/kubectl (v${kube_version}) install"
    multipass exec "${vm_name}" -- bash -c "${install_script}"
}

# VM 内で任意 bash コマンドを実行（pipe 越し / heredoc 越しの shell script に対応）
e2e_multipass_exec() {
    # 引数 1: VM 名 / 引数 2 以降: 実行コマンド
    local vm_name="$1"
    shift
    multipass exec "${vm_name}" -- "$@"
}

# VM の IPv4 を取得（kubeadm init の --apiserver-advertise-address 等で使う）
e2e_multipass_ip() {
    # 引数 1: VM 名
    local vm_name="$1"
    multipass info "${vm_name}" | awk '/IPv4/ {print $2; exit}'
}

# VM 一覧をまとめて削除（cleanup trap で呼ぶ）
e2e_multipass_delete_all() {
    # 引数: VM 名のリスト（可変長）
    local vms=("$@")
    if [[ ${#vms[@]} -eq 0 ]]; then
        return 0
    fi
    e2e_log "multipass VM 削除: ${vms[*]}"
    multipass delete --purge "${vms[@]}" 2>/dev/null || true
}

# VM が Running 状態か確認（check.sh で使う、不在 / Stopped で return 1）
e2e_multipass_running() {
    # 引数 1: VM 名
    local vm_name="$1"
    local state
    state="$(multipass info "${vm_name}" 2>/dev/null | awk '/State:/ {print $2; exit}')"
    if [[ "${state}" == "Running" ]]; then
        return 0
    fi
    return 1
}
