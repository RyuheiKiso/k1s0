#!/usr/bin/env bash
#
# tools/e2e/lib/kubeadm.sh — kubeadm cluster bootstrap helper（owner suite 専用）
#
# 設計正典:
#   ADR-TEST-008（owner suite: kubeadm 3CP HA + 2W）
#   ADR-INFRA-001（kubeadm + Cluster API 採用、本番経路）
#   docs/05_実装/30_CI_CD設計/35_e2e_test_design/10_owner_suite/01_環境契約.md
#
# 直接実行は不可。owner/up.sh から `source` で読み込む。
# 前提: tools/e2e/lib/{common,multipass}.sh が source 済
#
# 提供関数:
#   e2e_kubeadm_init_cp1            — control-plane 1 で kubeadm init + cert upload
#   e2e_kubeadm_join_cp             — 追加 control-plane を join（HA 構成）
#   e2e_kubeadm_join_worker         — worker を join
#   e2e_kubeadm_fetch_kubeconfig    — control-plane 1 から kubeconfig を取得
#   e2e_kubeadm_label_zone          — node に topology.kubernetes.io/zone label を付与

if [[ -n "${E2E_LIB_KUBEADM_LOADED:-}" ]]; then
    return 0
fi
readonly E2E_LIB_KUBEADM_LOADED=1

# control-plane 1 で kubeadm init を実行し、HA join 用の cert key を upload する。
# pod-network-cidr は Cilium 既定（10.244.0.0/16）と整合。
# 出力（標準出力）: 以下の 2 行（呼び出し側で eval / parse する）
#   JOIN_CP_CMD=<control-plane join コマンド全体>
#   JOIN_W_CMD=<worker join コマンド全体>
e2e_kubeadm_init_cp1() {
    # 引数 1: control-plane VM 名 / 引数 2: K8s バージョン
    local cp1_vm="$1"
    local kube_version="$2"
    local cp1_ip
    cp1_ip="$(e2e_multipass_ip "${cp1_vm}")"

    e2e_log "control-plane 1 (${cp1_vm}, ip=${cp1_ip}) で kubeadm init"
    # --upload-certs で 2 時間有効な certificate key を生成、HA join で再利用する
    multipass exec "${cp1_vm}" -- sudo kubeadm init \
        --kubernetes-version "v${kube_version}" \
        --pod-network-cidr=10.244.0.0/16 \
        --apiserver-advertise-address="${cp1_ip}" \
        --upload-certs

    # cert key を取得（HA control-plane join に使う）
    local cert_key
    cert_key="$(multipass exec "${cp1_vm}" -- sudo kubeadm init phase upload-certs --upload-certs 2>/dev/null | tail -1)"

    # worker join コマンド取得
    local worker_join_cmd
    worker_join_cmd="$(multipass exec "${cp1_vm}" -- sudo kubeadm token create --print-join-command 2>/dev/null)"
    if [[ -z "${worker_join_cmd}" ]]; then
        e2e_fail "kubeadm token create が失敗"
    fi

    # control-plane join コマンドは worker join コマンドに --control-plane と --certificate-key を付加
    local cp_join_cmd="${worker_join_cmd} --control-plane --certificate-key ${cert_key}"

    # 呼び出し側で eval する形式で出力（標準出力には他の log を出さない、呼び出し前に log した）
    echo "JOIN_CP_CMD=${cp_join_cmd}"
    echo "JOIN_W_CMD=${worker_join_cmd}"
}

# 追加 control-plane を join（HA 構成、cp-2 / cp-3 用）
# 呼び出し前に JOIN_CP_CMD を eval / 取得しておく
e2e_kubeadm_join_cp() {
    # 引数 1: 追加 CP の VM 名 / 引数 2: control-plane join コマンド全体
    local cp_vm="$1"
    local join_cmd="$2"

    e2e_log "  ${cp_vm} を control-plane に join（HA）"
    # bash -c で実行することで kubeadm join の複数引数を正しく展開
    multipass exec "${cp_vm}" -- sudo bash -c "${join_cmd}"
}

# worker を join（w-1 / w-2 用）
e2e_kubeadm_join_worker() {
    # 引数 1: worker VM 名 / 引数 2: worker join コマンド全体
    local w_vm="$1"
    local join_cmd="$2"

    e2e_log "  ${w_vm} を worker として join"
    multipass exec "${w_vm}" -- sudo bash -c "${join_cmd}"
}

# control-plane 1 から admin kubeconfig を取得し、ローカルファイルに保存
e2e_kubeadm_fetch_kubeconfig() {
    # 引数 1: control-plane 1 VM 名 / 引数 2: 出力先 kubeconfig path
    local cp1_vm="$1"
    local kubeconfig_path="$2"

    e2e_log "kubeconfig 取得: ${kubeconfig_path}"
    multipass exec "${cp1_vm}" -- sudo cat /etc/kubernetes/admin.conf > "${kubeconfig_path}"
    chmod 600 "${kubeconfig_path}"

    # context 名を k1s0-owner-e2e に書き換え（複数 cluster と並走時の衝突回避）
    # kubectl config rename-context で kubernetes-admin@kubernetes → k1s0-owner-e2e
    KUBECONFIG="${kubeconfig_path}" kubectl config rename-context \
        kubernetes-admin@kubernetes k1s0-owner-e2e 2>/dev/null || true
}

# node に topology.kubernetes.io/zone={a,b} label を付与（疑似 multi-AZ）
# worker 2 台にそれぞれ zone-a / zone-b を割り当てる
e2e_kubeadm_label_zone() {
    # 引数 1: kubeconfig path / 引数 2: worker 1 VM 名 / 引数 3: worker 2 VM 名
    local kubeconfig_path="$1"
    local w1_vm="$2"
    local w2_vm="$3"

    e2e_log "疑似 multi-AZ topology label 付与（zone-a / zone-b）"
    # multipass の VM 名と Kubernetes node 名は同一（kubeadm の hostname-override なし時）
    KUBECONFIG="${kubeconfig_path}" kubectl label node "${w1_vm}" \
        topology.kubernetes.io/zone=a --overwrite
    KUBECONFIG="${kubeconfig_path}" kubectl label node "${w2_vm}" \
        topology.kubernetes.io/zone=b --overwrite
}
