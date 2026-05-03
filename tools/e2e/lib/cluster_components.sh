#!/usr/bin/env bash
#
# tools/e2e/lib/cluster_components.sh — Cilium / Longhorn / MetalLB install helper
#
# 設計正典:
#   ADR-TEST-008（owner suite: Cilium + Longhorn + MetalLB の本番再現）
#   ADR-NET-001（production CNI = Cilium）
#   ADR-STOR-001（CSI = Longhorn）
#   ADR-STOR-002（LB = MetalLB）
#
# 直接実行は不可。owner/up.sh から `source` で読み込む。
# 前提: tools/e2e/lib/common.sh が source 済、KUBECONFIG が export 済
#
# 提供関数:
#   e2e_install_cilium    — Cilium CNI install（eBPF + Hubble、helm）
#   e2e_install_longhorn  — Longhorn CSI install（3 replica StorageClass、helm）
#   e2e_install_metallb   — MetalLB install（L2 mode、IP pool）
#
# バージョンは local-stack/up.sh と整合させる前提で、呼び出し側が env で指定する。
# 既定値は本ファイル内に記載（local-stack/up.sh の versions と同期）。

if [[ -n "${E2E_LIB_CLUSTER_COMPONENTS_LOADED:-}" ]]; then
    return 0
fi
readonly E2E_LIB_CLUSTER_COMPONENTS_LOADED=1

# Cilium install（owner suite production-fidelity CNI）
# helm chart 経由で eBPF mode + Hubble を有効化
e2e_install_cilium() {
    # 引数 1: Cilium chart version（既定 1.16.5、ADR-NET-001 production と整合）
    local cilium_version="${1:-1.16.5}"

    e2e_log "Cilium CNI install (chart v${cilium_version}, eBPF + Hubble)"
    # cilium chart repo を追加（既存なら update のみ）
    helm repo add cilium https://helm.cilium.io/ 2>/dev/null || true
    helm repo update cilium

    # cilium namespace を作成（kubeadm init 直後は無いので明示作成）
    kubectl create namespace cilium-system --dry-run=client -o yaml | kubectl apply -f -

    # Cilium install（kubeadm cluster の API server を auto-detect）
    # eBPF host routing + Hubble (UI 込み) を有効化
    helm upgrade --install cilium cilium/cilium \
        --version "${cilium_version}" \
        --namespace cilium-system \
        --set kubeProxyReplacement=true \
        --set hubble.enabled=true \
        --set hubble.relay.enabled=true \
        --set hubble.ui.enabled=true \
        --set ipam.mode=cluster-pool \
        --set ipam.operator.clusterPoolIPv4PodCIDRList="10.244.0.0/16" \
        --wait --timeout=10m

    # cilium-system の全 Pod Ready 待機（control-plane Ready の前提）
    e2e_wait_pods_ready cilium-system 600
}

# Longhorn install（owner suite production-fidelity CSI）
# 3 replica StorageClass を default に設定
e2e_install_longhorn() {
    # 引数 1: Longhorn chart version（既定 1.7.2、ADR-STOR-001 と整合）
    local longhorn_version="${1:-1.7.2}"

    e2e_log "Longhorn CSI install (chart v${longhorn_version}, 3 replica)"
    # longhorn chart repo を追加
    helm repo add longhorn https://charts.longhorn.io 2>/dev/null || true
    helm repo update longhorn

    # longhorn-system namespace を作成
    kubectl create namespace longhorn-system --dry-run=client -o yaml | kubectl apply -f -

    # Longhorn install（3 replica + default StorageClass）
    # kubeadm + multipass VM 環境では open-iscsi の事前 install が必要（VM 起動時に install 済）
    helm upgrade --install longhorn longhorn/longhorn \
        --version "${longhorn_version}" \
        --namespace longhorn-system \
        --set persistence.defaultClassReplicaCount=3 \
        --set defaultSettings.defaultReplicaCount=3 \
        --wait --timeout=10m

    # default StorageClass annotation を longhorn に切り替える（k1s0-default が無い owner cluster）
    kubectl patch storageclass longhorn -p \
        '{"metadata":{"annotations":{"storageclass.kubernetes.io/is-default-class":"true"}}}'

    # longhorn-system Pod Ready 待機
    e2e_wait_pods_ready longhorn-system 600
}

# MetalLB install（owner suite L2 mode、IP pool 192.168.64.200-220）
# kind の extraPortMappings ではなく実 LoadBalancer Service を提供する production-fidelity 経路
e2e_install_metallb() {
    # 引数 1: MetalLB chart version（既定 v0.14.9、local-stack と整合）
    local metallb_version="${1:-v0.14.9}"
    # 引数 2: IP pool 開始（既定 192.168.64.200）
    local pool_start="${2:-192.168.64.200}"
    # 引数 3: IP pool 終了（既定 192.168.64.220）
    local pool_end="${3:-192.168.64.220}"

    e2e_log "MetalLB install (${metallb_version}, L2 mode, pool ${pool_start}-${pool_end})"
    # MetalLB native manifest（helm でも可だが native が軽量）
    kubectl apply -f \
        "https://raw.githubusercontent.com/metallb/metallb/${metallb_version}/config/manifests/metallb-native.yaml"

    # webhook が起動するまで待機（最大 5 分）
    e2e_wait_pods_ready metallb-system 300

    # IPAddressPool + L2Advertisement を apply
    kubectl apply -f - <<EOF
apiVersion: metallb.io/v1beta1
kind: IPAddressPool
metadata:
  name: k1s0-owner-e2e-pool
  namespace: metallb-system
spec:
  addresses:
    - ${pool_start}-${pool_end}
---
apiVersion: metallb.io/v1beta1
kind: L2Advertisement
metadata:
  name: k1s0-owner-e2e-l2
  namespace: metallb-system
spec:
  ipAddressPools:
    - k1s0-owner-e2e-pool
EOF
}
