#!/usr/bin/env bash
# D 軸: OSS 完成度チェック
#
# 判定基準: docs/00_format/audit_criteria.md §D 軸
# 出力: ${EVIDENCE_DIR}/oss-checklist.txt
#
# 設計原則:
#   - ルートファイル存在チェックは機械化（簡単）
#   - OSSF Scorecard は scorecard-cli が必要、不在なら保留
#   - 各項目に Met / Unmet / N/A を明記
#   - 判定は人間が下す、本 script は証跡を並べるのみ

set -euo pipefail
REPO_ROOT="$1"
EVIDENCE_DIR="$2"

OSS_OUT="${EVIDENCE_DIR}/oss-checklist.txt"

{
  echo "# OSS 完成度チェックリスト (生成: $(date -Iseconds))"
  echo
  echo "## CNCF Sandbox 最低要件 ファイル存在"
  echo
} > "${OSS_OUT}"

check_file() {
  local path="$1"
  local label="$2"
  if [[ -f "${REPO_ROOT}/${path}" ]]; then
    local lines
    lines="$(wc -l < "${REPO_ROOT}/${path}" | tr -d ' ')"
    echo "- [${label}] Met: ${path} (${lines} lines)" >> "${OSS_OUT}"
  else
    echo "- [${label}] Unmet: ${path} 不在" >> "${OSS_OUT}"
  fi
}

check_file "LICENSE" "LICENSE"
check_file "CODE_OF_CONDUCT.md" "Code of Conduct"
check_file "CONTRIBUTING.md" "Contributing Guide"
check_file "GOVERNANCE.md" "Governance"
check_file "SECURITY.md" "Security Policy"
check_file "README.md" "README"

{
  echo
  echo "## OpenSSF Best Practices Badge — Passing 主要項目（手動チェック必須）"
  echo
  echo "- [Basics] 公開 repo / OSS license / 説明文 / VCS 利用 — README / LICENSE 存在で部分的に満たす"
  echo "- [Change Control] 公開 VCS / 半年以上の history / 一意なバージョン番号 — git log で確認"
  echo "- [Reporting] vulnerability 報告経路 — SECURITY.md で明示"
  echo "- [Quality] build / test / continuous integration — Makefile / .github/workflows/ で確認"
  echo "- [Security] cryptography 適切利用 / hardening — 個別 ADR 参照"
  echo "- [Analysis] static analyzer / warnings — tools/ci/ 参照"
  echo
  echo "詳細は外部 https://www.bestpractices.dev/ で repo URL を入力して採点"
} >> "${OSS_OUT}"

# OSSF Scorecard
{
  echo
  echo "## OSSF Scorecard"
  echo
} >> "${OSS_OUT}"

if command -v scorecard >/dev/null 2>&1 || command -v scorecard-cli >/dev/null 2>&1; then
  echo "scorecard CLI 検出。実行は別途必要（公開 repo URL を引数で渡す）" >> "${OSS_OUT}"
else
  echo "status: scorecard_not_installed" >> "${OSS_OUT}"
  echo "note: scorecard-cli を導入後に手動実行 / GitHub Action で自動採点" >> "${OSS_OUT}"
fi

# git history からメンテ頻度を出す
{
  echo
  echo "## メンテ活発度（git log 直近 30 日）"
  echo
  cd "${REPO_ROOT}" 2>/dev/null && {
    commits_30d="$(git log --since='30 days ago' --oneline 2>/dev/null | wc -l | tr -d ' ' || echo 0)"
    contributors_all="$(git log --format='%ae' 2>/dev/null | sort -u | wc -l | tr -d ' ' || echo 0)"
    echo "commits_last_30_days: ${commits_30d}"
    echo "unique_contributors_all_time: ${contributors_all}"
  } || echo "git log 取得不可"
} >> "${OSS_OUT}"

echo "=== oss 軸 ==="
cat "${OSS_OUT}"
