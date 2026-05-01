#!/usr/bin/env bash
# =============================================================================
# ops/scripts/lib/common.sh — POSIX sh 共通ライブラリ
#
# 役割: ops/scripts/*.sh および ops/dr/scripts/*.sh から source される共通関数集約
# =============================================================================

# UTC タイムスタンプ（ファイル名・ログ用）
k1s0_utc_timestamp() {
    date -u +%Y%m%dT%H%M%SZ
}

# ISO 8601 タイムスタンプ
k1s0_iso_timestamp() {
    date -u +%Y-%m-%dT%H:%M:%SZ
}

# ログ出力（stdout + ログファイル）
# Usage: k1s0_log info "メッセージ"
k1s0_log() {
    local lvl="$1"; shift
    printf '[%s] [%s] %s\n' "$(k1s0_iso_timestamp)" "${lvl}" "$*"
    [[ -n "${LOG_FILE:-}" ]] && \
        printf '[%s] [%s] %s\n' "$(k1s0_iso_timestamp)" "${lvl}" "$*" >> "${LOG_FILE}"
}

# 必須ツールの存在確認
# Usage: k1s0_need kubectl argocd
k1s0_need() {
    local missing=()
    for cmd in "$@"; do
        if ! command -v "${cmd}" >/dev/null 2>&1; then
            missing+=("${cmd}")
        fi
    done
    if [[ ${#missing[@]} -gt 0 ]]; then
        k1s0_log error "必須ツール不在: ${missing[*]}"
        return 1
    fi
}

# kubectl context が prod かを確認（誤実行防止）
# Usage: k1s0_assert_prod
k1s0_assert_prod() {
    local ctx
    ctx=$(kubectl config current-context 2>/dev/null || echo "<none>")
    if [[ "${ctx}" != "k1s0-prod" ]]; then
        k1s0_log error "本番 context ではない (current=${ctx})"
        k1s0_log error "本スクリプトは prod でのみ実行可能。--dry-run で staging テストを"
        return 1
    fi
}

# Slack 通知（SLACK_WEBHOOK_URL 環境変数または無視）
# Usage: k1s0_slack "message"
k1s0_slack() {
    local msg="$1"
    if [[ -z "${SLACK_WEBHOOK_URL:-}" ]]; then
        k1s0_log warn "SLACK_WEBHOOK_URL 未設定、Slack 通知スキップ"
        return 0
    fi
    curl -sS -X POST -H 'Content-type: application/json' \
        --data "{\"text\":\"${msg}\"}" "${SLACK_WEBHOOK_URL}" >/dev/null
}

# 排他ロック（並列実行禁止）
# Usage: k1s0_lock "operation-name"
# trap 'k1s0_unlock "..."' EXIT で解除
k1s0_lock() {
    local name="$1"
    local lock_dir="${TMPDIR:-/tmp}/k1s0-lock.${name}"
    if ! mkdir "${lock_dir}" 2>/dev/null; then
        k1s0_log error "並列実行禁止: ${lock_dir} 存在 (他プロセス進行中？)"
        return 1
    fi
    echo "${lock_dir}"
}

k1s0_unlock() {
    local name="$1"
    rmdir "${TMPDIR:-/tmp}/k1s0-lock.${name}" 2>/dev/null || true
}

# 経過時間（秒 / 分）
# Usage: K1S0_START=$(date +%s); ...; k1s0_elapsed_min "${K1S0_START}"
k1s0_elapsed_sec() {
    local start="$1"
    echo $(( $(date +%s) - start ))
}

k1s0_elapsed_min() {
    local start="$1"
    awk -v s="${start}" -v n="$(date +%s)" 'BEGIN { printf "%.1f", (n-s)/60 }'
}
