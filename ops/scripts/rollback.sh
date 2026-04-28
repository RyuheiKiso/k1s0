#!/usr/bin/env bash
# =============================================================================
# ops/scripts/rollback.sh — 1 コマンド rollback (15 分以内 SLA)
#
# 設計: docs/05_実装/70_リリース設計/ 配下の rollback runbook
# 関連 ID:
#   IMP-REL-RB-050: 5 段階タイムライン定義
#   IMP-REL-RB-051: 1 コマンド化 + GitHub OIDC 認証
#   IMP-REL-RB-052: Branch Protection による SRE 承認強制 (4-eyes)
#   IMP-REL-RB-053: Helm sync + Rollout undo 並列実行
#   IMP-REL-RB-059: Incident メタデータ記録
# 関連 NFR: NFR-A-FT-001 (自動復旧 15 分以内), NFR-C-IR-001 (Incident Response)
#
# 役割:
#   tier1/tier2/tier3 のいずれかのデプロイ済みアプリを直前の良好版へ即座に
#   切り戻すための一括スクリプト。Phase 2〜3（Git revert → Argo CD sync &
#   Rollouts undo 並列実行）を 8 分以内に完了させる。
#
# 5 段階タイムライン (合計 15 分):
#   Phase 1 (2分) — 検知: SRE オンコール起動・自動 vs 手動判定
#   Phase 2 (3分) — Git revert: revert PR 生成・SRE 承認・main マージ
#   Phase 3 (5分) — Argo sync: app sync + Rollouts undo の並列実行
#   Phase 4 (5分) — AnalysisTemplate 5 本評価 (本スクリプト外)
#   Phase 5 (継続) — 安定化観測 + Postmortem PR 生成 (本スクリプト外)
#
# Usage:
#   ops/scripts/rollback.sh \
#     --app <argo-app-name> \
#     --revision <prev-good-sha-or-tag> \
#     --reason "<incident-description>" \
#     [--incident <pagerduty-id>] \
#     [--dry-run]
#
# 環境変数:
#   GITHUB_TOKEN     — gh CLI 認証 (CI では OIDC で自動)
#   ARGOCD_SERVER    — Argo CD API エンドポイント
#   ARGOCD_AUTH_TOKEN — Argo CD 認証 (gpg-agent or kubernetes secret)
#   PAGERDUTY_TOKEN  — Incident 添付用 (空なら添付スキップ)
#   K1S0_ROLLBACK_REPO — 既定 "k1s0/k1s0"
#
# Exit code:
#   0 — Phase 2〜3 完了 (rollback 成功)
#   1 — 検証失敗 / lock 競合 / SRE 承認タイムアウト
#   2 — 引数エラー / 必須ツール不在
#
# 制約:
#   並列実行禁止 (Incident context manager による排他制御を前提)
#   SLO breach (15 分超過) は Incident メタデータに記録
# =============================================================================
set -euo pipefail

# ----------------------------------------------------------------------------
# 引数解析
# ----------------------------------------------------------------------------
APP=""
REVISION=""
REASON=""
INCIDENT=""
DRY_RUN=0

usage() {
    sed -n '3,40p' "$0" | sed 's/^# \{0,1\}//'
    exit 2
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --app) APP="$2"; shift 2 ;;
        --revision) REVISION="$2"; shift 2 ;;
        --reason) REASON="$2"; shift 2 ;;
        --incident) INCIDENT="$2"; shift 2 ;;
        --dry-run) DRY_RUN=1; shift ;;
        -h|--help) usage ;;
        *)
            echo "[error] 未知のオプション: $1" >&2
            usage
            ;;
    esac
done

# 必須引数の検証
if [[ -z "${APP}" || -z "${REVISION}" || -z "${REASON}" ]]; then
    echo "[error] --app / --revision / --reason は必須" >&2
    usage
fi

# Argo CD アプリ名の単純検証 (英数 + ハイフン)
if ! [[ "${APP}" =~ ^[a-z0-9][a-z0-9-]*$ ]]; then
    echo "[error] --app は英小文字・数字・ハイフンのみ: ${APP}" >&2
    exit 2
fi

# revision の単純検証 (40-hex sha or タグ風)
if ! [[ "${REVISION}" =~ ^([0-9a-f]{7,40}|v[0-9].*)$ ]]; then
    echo "[error] --revision は 7-40 桁 sha または v 始まりタグ: ${REVISION}" >&2
    exit 2
fi

# ----------------------------------------------------------------------------
# 必須ツールの存在チェック
# ----------------------------------------------------------------------------
need() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "[error] 必須ツール不在: $1 (--dry-run 時はスキップ)" >&2
        return 1
    fi
}

# Phase 別に必要なツール
TOOLS_PHASE2=(git gh)
TOOLS_PHASE3=(argocd kubectl)

if [[ "${DRY_RUN}" == "0" ]]; then
    for t in "${TOOLS_PHASE2[@]}" "${TOOLS_PHASE3[@]}"; do
        need "$t" || exit 2
    done
fi

# ----------------------------------------------------------------------------
# ロギング (PagerDuty Incident への添付想定)
# ----------------------------------------------------------------------------
LOG_DIR="${TMPDIR:-/tmp}/k1s0-rollback"
mkdir -p "${LOG_DIR}"
LOG_FILE="${LOG_DIR}/${APP}-$(date -u +%Y%m%dT%H%M%SZ).log"
START_EPOCH="$(date +%s)"

log() {
    local level="$1"; shift
    local ts
    ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    printf '[%s] [%s] %s\n' "${ts}" "${level}" "$*" | tee -a "${LOG_FILE}"
}

elapsed_min() {
    local now_epoch
    now_epoch="$(date +%s)"
    awk -v s="${START_EPOCH}" -v n="${now_epoch}" 'BEGIN { printf "%.1f", (n-s)/60 }'
}

# ----------------------------------------------------------------------------
# Phase 0: 排他ロック (並列 rollback 禁止)
# ----------------------------------------------------------------------------
LOCK_DIR="${TMPDIR:-/tmp}/k1s0-rollback.lock.${APP}"
if ! mkdir "${LOCK_DIR}" 2>/dev/null; then
    log error "並列 rollback 禁止: ${LOCK_DIR} 存在 (他の rollback 進行中？)"
    exit 1
fi
trap 'rmdir "${LOCK_DIR}" 2>/dev/null || true' EXIT

log info "rollback 開始: app=${APP} revision=${REVISION} dry_run=${DRY_RUN}"
log info "reason: ${REASON}"
[[ -n "${INCIDENT}" ]] && log info "incident: ${INCIDENT}"

# ----------------------------------------------------------------------------
# Phase 2 (3 分): Git revert → PR 生成 → SRE 承認 → main マージ
# ----------------------------------------------------------------------------
phase2_git_revert() {
    log info "Phase 2 開始 (Git revert + PR + SRE 承認 + マージ)"

    local repo="${K1S0_ROLLBACK_REPO:-k1s0/k1s0}"
    local branch="rollback/${APP}-$(date -u +%Y%m%dT%H%M%SZ)"
    local title="revert(${APP}): rollback to ${REVISION} — ${REASON}"

    if [[ "${DRY_RUN}" == "1" ]]; then
        log info "[dry-run] git checkout -b ${branch}"
        log info "[dry-run] git revert --no-edit <commits between ${REVISION}..HEAD>"
        log info "[dry-run] git push origin ${branch}"
        log info "[dry-run] gh pr create --title '${title}' --base main --label 'rollback,incident'"
        log info "[dry-run] gh pr review --approve (require 2nd SRE in real path)"
        log info "[dry-run] gh pr merge --squash"
        return 0
    fi

    # 既存の作業ツリーを汚さないため、別ブランチで実施
    git fetch --quiet origin main
    git checkout -B "${branch}" origin/main >>"${LOG_FILE}" 2>&1

    # ${REVISION}..HEAD の各 commit を順番に revert
    if ! git revert --no-edit "${REVISION}..HEAD" >>"${LOG_FILE}" 2>&1; then
        log error "git revert 失敗 — マージコミット混在の可能性。手動 revert に切替を"
        return 1
    fi

    git push --quiet -u origin "${branch}"

    # PR 生成 (Branch Protection で 4-eyes が強制される前提)
    local pr_url
    pr_url="$(gh pr create \
        --repo "${repo}" \
        --base main \
        --head "${branch}" \
        --title "${title}" \
        --body "Automated rollback by ops/scripts/rollback.sh
revision: ${REVISION}
reason: ${REASON}
incident: ${INCIDENT:-N/A}
log: ${LOG_FILE}" \
        --label rollback \
        --label incident 2>>"${LOG_FILE}")"
    log info "PR 作成: ${pr_url}"

    # SRE 承認待ち (最大 5 分)
    local pr_num="${pr_url##*/}"
    local approved=0
    local i
    for i in $(seq 1 30); do
        local state
        state="$(gh pr view "${pr_num}" --repo "${repo}" --json reviewDecision --jq '.reviewDecision')"
        if [[ "${state}" == "APPROVED" ]]; then
            approved=1
            break
        fi
        log info "SRE 承認待ち (${i}/30, ${state:-pending})"
        sleep 10
    done

    if [[ "${approved}" == "0" ]]; then
        log error "SRE 承認タイムアウト (5 分)"
        return 1
    fi

    # マージ
    gh pr merge "${pr_num}" --repo "${repo}" --squash --delete-branch >>"${LOG_FILE}" 2>&1
    log info "main にマージ完了"
    return 0
}

# ----------------------------------------------------------------------------
# Phase 3 (5 分): Argo CD sync + Rollouts undo を並列実行
# ----------------------------------------------------------------------------
phase3_sync_and_undo() {
    log info "Phase 3 開始 (Argo CD sync + Rollouts undo 並列)"

    if [[ "${DRY_RUN}" == "1" ]]; then
        log info "[dry-run] argocd app sync ${APP} --revision main --prune"
        log info "[dry-run] kubectl argo rollouts undo ${APP} (並列)"
        log info "[dry-run] argocd app wait ${APP} --health (max 5 分)"
        return 0
    fi

    # Argo CD sync を bg で起動
    (
        argocd app sync "${APP}" \
            --revision main \
            --prune \
            --timeout 300 >>"${LOG_FILE}" 2>&1
    ) &
    local pid_sync=$!

    # Rollouts undo を並列で起動 (rollout が存在しないアプリは noop で抜ける想定)
    (
        kubectl argo rollouts undo "${APP}" \
            --namespace "${APP}" \
            --to-revision=0 >>"${LOG_FILE}" 2>&1 || true
    ) &
    local pid_undo=$!

    # 両方の終了を待機
    local rc_sync=0 rc_undo=0
    wait "${pid_sync}" || rc_sync=$?
    wait "${pid_undo}" || rc_undo=$?

    if [[ "${rc_sync}" -ne 0 ]]; then
        log error "argocd app sync 失敗 (exit ${rc_sync})"
        return 1
    fi
    log info "argocd app sync 完了 (rollouts undo exit ${rc_undo} — 0/非0 ともに rollout 不在は許容)"

    # health 待機 (最大 5 分)
    if ! argocd app wait "${APP}" --health --timeout 300 >>"${LOG_FILE}" 2>&1; then
        log error "argocd app wait health タイムアウト"
        return 1
    fi
    log info "Argo CD app health=Healthy 確認"
    return 0
}

# ----------------------------------------------------------------------------
# Incident メタデータ記録 (IMP-REL-RB-059)
# ----------------------------------------------------------------------------
record_incident_metadata() {
    local status="$1"  # "success" | "failed"
    local meta_file="${LOG_DIR}/${APP}-$(date -u +%Y%m%dT%H%M%SZ).meta.json"
    local duration_min
    duration_min="$(elapsed_min)"
    local breach="false"
    awk -v d="${duration_min}" 'BEGIN { exit !(d > 15) }' && breach="true"

    cat >"${meta_file}" <<EOF
{
  "app": "${APP}",
  "revision": "${REVISION}",
  "reason": "${REASON}",
  "incident": "${INCIDENT}",
  "status": "${status}",
  "rollback_duration_minutes": ${duration_min},
  "slo_breach_15min": ${breach},
  "log_file": "${LOG_FILE}",
  "dry_run": $([ "${DRY_RUN}" -eq 1 ] && echo true || echo false),
  "started_at": "$(date -u -d "@${START_EPOCH}" +%Y-%m-%dT%H:%M:%SZ)",
  "ended_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
    log info "Incident メタデータ記録: ${meta_file}"

    # PagerDuty への添付 (token が無ければスキップ)
    if [[ -n "${PAGERDUTY_TOKEN:-}" && -n "${INCIDENT}" && "${DRY_RUN}" == "0" ]]; then
        log info "PagerDuty Incident ${INCIDENT} へログ添付 (TODO: API 呼び出し)"
    fi
}

# ----------------------------------------------------------------------------
# main
# ----------------------------------------------------------------------------
main() {
    if ! phase2_git_revert; then
        record_incident_metadata "failed"
        log error "Phase 2 失敗 — rollback 中断"
        exit 1
    fi

    if ! phase3_sync_and_undo; then
        record_incident_metadata "failed"
        log error "Phase 3 失敗 — Argo CD 経路を確認"
        exit 1
    fi

    record_incident_metadata "success"
    local d
    d="$(elapsed_min)"
    log info "rollback 完了 (Phase 2+3, 経過時間 ${d} 分)"
    if awk -v d="${d}" 'BEGIN { exit !(d > 8) }'; then
        log warn "Phase 2+3 で 8 分超過 — Phase 4-5 を含む 15 分 SLA に注意"
    fi
}

main
