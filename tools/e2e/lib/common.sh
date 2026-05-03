#!/usr/bin/env bash
#
# tools/e2e/lib/common.sh — owner / user 両 suite 共通ヘルパー
#
# 設計正典:
#   ADR-TEST-008（e2e owner / user 二分構造）
#   docs/05_実装/30_CI_CD設計/35_e2e_test_design/00_方針/01_owner_user_責務分界.md
#
# 直接実行は不可。owner/up.sh / user/up.sh から `source` で読み込む。
# 提供関数:
#   e2e_log / e2e_warn / e2e_fail   — 統一ログ出力
#   e2e_repo_root                    — git repo root 取得
#   e2e_run_date                     — 実走日（YYYY-MM-DD UTC）
#   e2e_owner_artifact_dir           — owner 実走 artifact 出力先
#   e2e_user_artifact_dir            — user 実走 artifact 出力先
#   e2e_require_bin                  — 必須 binary 存在確認
#   e2e_kubectl_context_check        — kubeconfig context 一致確認
#   e2e_wait_nodes_ready             — 全 node Ready 待機
#   e2e_wait_pods_ready              — namespace 内全 Pod Ready 待機
#   e2e_sha256                       — file の sha256sum 計算

# 二重 source 防止（再入時に readonly 衝突を避ける）
if [[ -n "${E2E_LIB_COMMON_LOADED:-}" ]]; then
    return 0
fi
readonly E2E_LIB_COMMON_LOADED=1

# 統一ログ出力（local-stack/up.sh の log/warn/fail と意図的に異なる prefix で
# どの layer のログか即判別できるようにする）
e2e_log() {
    # 通常情報出力（青）
    printf '\033[36m[e2e]\033[0m %s\n' "$*"
}

e2e_warn() {
    # 警告（黄）
    printf '\033[33m[e2e][warn]\033[0m %s\n' "$*"
}

e2e_fail() {
    # 致命的エラー（赤）と exit 1
    printf '\033[31m[e2e][error]\033[0m %s\n' "$*" >&2
    exit 1
}

# git repo root を取得（cd-safe、再帰呼び出しで使う前提）
e2e_repo_root() {
    git rev-parse --show-toplevel 2>/dev/null || pwd
}

# 実走日（UTC、YYYY-MM-DD）。同日複数回実走時は呼び出し側で時刻 suffix を付ける
e2e_run_date() {
    date -u +%Y-%m-%d
}

# owner 実走 artifact 出力先（ADR-TEST-011 の git LFS 12 ヶ月管理対象）
e2e_owner_artifact_dir() {
    # 引数 1: 実走日（指定なしなら e2e_run_date）
    local run_date="${1:-$(e2e_run_date)}"
    echo "$(e2e_repo_root)/tests/.owner-e2e/${run_date}"
}

# user 実走 artifact 出力先（CI で 14 日 retention、local では ad-hoc）
e2e_user_artifact_dir() {
    local run_date="${1:-$(e2e_run_date)}"
    echo "$(e2e_repo_root)/tests/.user-e2e/${run_date}"
}

# 必須 binary 存在確認（不在なら install 案内を表示して呼び出し側に return 1）
e2e_require_bin() {
    # 引数 1: binary 名 / 引数 2: install 案内文
    local bin_name="$1"
    local install_hint="${2:-}"
    if ! command -v "${bin_name}" >/dev/null 2>&1; then
        e2e_warn "${bin_name} not found in PATH"
        if [[ -n "${install_hint}" ]]; then
            e2e_warn "  install: ${install_hint}"
        fi
        return 1
    fi
    return 0
}

# kubectl の current-context が期待値と一致するか確認
# 不一致の場合は warning 出力 + return 1（呼び出し側で fail させるか判断）
e2e_kubectl_context_check() {
    # 引数 1: 期待 context 名（例: k1s0-owner-e2e / kind-k1s0-user-e2e）
    local expected_ctx="$1"
    local current_ctx
    current_ctx="$(kubectl config current-context 2>/dev/null || echo "")"
    if [[ "${current_ctx}" != "${expected_ctx}" ]]; then
        e2e_warn "kubeconfig context 不一致: expected=${expected_ctx} / current=${current_ctx}"
        return 1
    fi
    return 0
}

# 全 node が Ready になるまで待機（最大 timeout 秒、既定 300 秒）
# 失敗時は kubectl describe で diag を吐いて return 1
e2e_wait_nodes_ready() {
    # 引数 1: timeout 秒（既定 300）
    local timeout_sec="${1:-300}"
    e2e_log "全 node Ready 待機（最大 ${timeout_sec} 秒）"
    if kubectl wait --for=condition=Ready node --all --timeout="${timeout_sec}s"; then
        e2e_log "全 node Ready 確認"
        return 0
    fi
    e2e_warn "node Ready 待機タイムアウト、kubectl describe で diag 出力"
    kubectl get nodes -o wide
    kubectl describe nodes
    return 1
}

# 指定 namespace 内の全 Pod が Ready になるまで待機
# Job / Completed Pod は除外（kubectl wait の仕様）
e2e_wait_pods_ready() {
    # 引数 1: namespace / 引数 2: timeout 秒（既定 300）
    local namespace="$1"
    local timeout_sec="${2:-300}"
    e2e_log "namespace=${namespace} の全 Pod Ready 待機（最大 ${timeout_sec} 秒）"
    if kubectl wait --for=condition=Ready pod --all --namespace="${namespace}" --timeout="${timeout_sec}s" 2>/dev/null; then
        e2e_log "namespace=${namespace} 全 Pod Ready"
        return 0
    fi
    e2e_warn "namespace=${namespace} Pod Ready 待機タイムアウト"
    kubectl get pods -n "${namespace}" -o wide
    return 1
}

# file の sha256sum を 64 文字 HEX で出力（cut.sh の検証経路と整合）
e2e_sha256() {
    # 引数 1: ファイルパス
    local file_path="$1"
    if [[ ! -f "${file_path}" ]]; then
        e2e_warn "sha256 対象ファイル不在: ${file_path}"
        return 1
    fi
    sha256sum "${file_path}" | cut -d' ' -f1
}
