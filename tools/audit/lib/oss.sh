#!/usr/bin/env bash
# D 軸: OSS 完成度チェック
#
# 判定基準: docs/00_format/audit_criteria.md §D 軸
# 採点基準: .claude/skills/oss-completeness-criteria/SKILL.md（OSSF Scorecard 18 項目 / CNCF Sandbox / OpenSSF Best Practices）
# 出力: ${EVIDENCE_DIR}/oss-checklist.txt
#
# 設計原則 (audit-protocol skill 準拠):
#   - 機械的に検証可能な項目のみ自動採点（Met / Unmet / Unknown / N/A の 4 値）
#   - public repo / scorecard-cli が前提の項目は Unknown と明記（嘘 PASS を書かない）
#   - 各項目に証跡パス（grep 結果 / find 結果 / ファイル参照）を併記
#   - 判定総合は人間が AUDIT.md で行う

set -euo pipefail
REPO_ROOT="$1"
EVIDENCE_DIR="$2"

OSS_OUT="${EVIDENCE_DIR}/oss-checklist.txt"

# 安全な count helper（grep -c の "0\n0" 罠回避）
safe_count() {
  local n
  n="$(eval "$1" 2>/dev/null || true)"
  echo "${n:-0}"
}

{
  echo "# OSS 完成度チェックリスト (生成: $(date -Iseconds))"
  echo "# 採点基準: OSSF Scorecard / CNCF Sandbox / OpenSSF Best Practices Badge"
  echo "# 判定値: Met / Unmet / Unknown(public repo + scorecard-cli が前提) / N/A"
  echo
  echo "## CNCF Sandbox 最低要件 (ファイル存在 + 中身の最低要素)"
  echo
} > "${OSS_OUT}"

check_file() {
  local path="$1"
  local label="$2"
  if [[ -f "${REPO_ROOT}/${path}" ]]; then
    local lines
    lines="$(wc -l < "${REPO_ROOT}/${path}" | tr -d ' ')"
    echo "- [${label}] Met: ${path} (${lines} lines)" >> "${OSS_OUT}"
    return 0
  else
    echo "- [${label}] Unmet: ${path} 不在" >> "${OSS_OUT}"
    return 1
  fi
}

check_file "LICENSE" "LICENSE"
check_file "CODE_OF_CONDUCT.md" "Code of Conduct"
check_file "CONTRIBUTING.md" "Contributing Guide"
check_file "GOVERNANCE.md" "Governance"
check_file "SECURITY.md" "Security Policy"
check_file "README.md" "README"

# LICENSE の種別判定 (OSI 承認 license の識別)
{
  echo
  echo "## LICENSE 種別判定 (OSI 承認 license)"
  echo
} >> "${OSS_OUT}"
if [[ -f "${REPO_ROOT}/LICENSE" ]]; then
  license_type="Unknown"
  # head -5 の複数行を 1 行に集約してから判定（Apache License と Version 2.0 が別行）
  license_head_joined="$(head -5 "${REPO_ROOT}/LICENSE" | tr -s '[:space:]' ' ')"
  if echo "${license_head_joined}" | grep -qiE "Apache License.*Version 2\.0"; then
    license_type="Apache-2.0 (OSI 承認)"
  elif echo "${license_head_joined}" | grep -qiE "MIT License"; then
    license_type="MIT (OSI 承認)"
  elif echo "${license_head_joined}" | grep -qiE "GNU AFFERO GENERAL PUBLIC LICENSE"; then
    license_type="AGPL-3.0 (OSI 承認、強い copyleft)"
  elif echo "${license_head_joined}" | grep -qiE "GNU GENERAL PUBLIC LICENSE"; then
    license_type="GPL (OSI 承認、copyleft)"
  elif echo "${license_head_joined}" | grep -qiE "BSD"; then
    license_type="BSD 系 (OSI 承認)"
  elif echo "${license_head_joined}" | grep -qiE "Mozilla Public License"; then
    license_type="MPL-2.0 (OSI 承認)"
  fi
  echo "- 識別結果: ${license_type}" >> "${OSS_OUT}"
  echo "  証跡: LICENSE 冒頭 5 行を grep で識別" >> "${OSS_OUT}"
else
  echo "- LICENSE ファイル不在 (Unmet)" >> "${OSS_OUT}"
fi

# SECURITY.md 内の vulnerability 報告経路
{
  echo
  echo "## Security-Policy (Scorecard 項目)"
  echo
} >> "${OSS_OUT}"
if [[ -f "${REPO_ROOT}/SECURITY.md" ]]; then
  has_mailto=$(safe_count "grep -c -E 'mailto:|@' '${REPO_ROOT}/SECURITY.md'")
  has_url=$(safe_count "grep -c -E 'https?://' '${REPO_ROOT}/SECURITY.md'")
  has_disclosure=$(safe_count "grep -c -iE 'disclosure|報告|report|coordinated' '${REPO_ROOT}/SECURITY.md'")
  if [[ "${has_mailto}" -gt 0 || "${has_url}" -gt 0 ]] && [[ "${has_disclosure}" -gt 0 ]]; then
    echo "- [Vulnerability 報告経路] Met: mailto/URL ${has_mailto}/${has_url} 件、disclosure キーワード ${has_disclosure} 件" >> "${OSS_OUT}"
  else
    echo "- [Vulnerability 報告経路] Unmet: SECURITY.md は存在するが報告経路が不明瞭" >> "${OSS_OUT}"
  fi
else
  echo "- SECURITY.md 不在 (Unmet)" >> "${OSS_OUT}"
fi

# .github/ 系 (リポジトリ運用の整備度)
{
  echo
  echo "## リポジトリ運用 (.github/ 系)"
  echo
} >> "${OSS_OUT}"
check_file ".github/CODEOWNERS" "CODEOWNERS"
check_file ".github/PULL_REQUEST_TEMPLATE.md" "PR Template"
check_file ".github/labels.yml" "Labels Definition"
check_file ".github/repo-settings.md" "Repo Settings (文書化)"
if [[ -d "${REPO_ROOT}/.github/ISSUE_TEMPLATE" ]]; then
  n_templates=$(find "${REPO_ROOT}/.github/ISSUE_TEMPLATE" -maxdepth 1 -type f -name '*.md' -o -name '*.yml' 2>/dev/null | wc -l | tr -d ' ')
  echo "- [Issue Template] Met: .github/ISSUE_TEMPLATE/ (${n_templates} 件)" >> "${OSS_OUT}"
else
  echo "- [Issue Template] Unmet: .github/ISSUE_TEMPLATE/ 不在" >> "${OSS_OUT}"
fi

# CI-Tests (Scorecard 項目)
{
  echo
  echo "## CI-Tests (Scorecard 項目)"
  echo
} >> "${OSS_OUT}"
if [[ -d "${REPO_ROOT}/.github/workflows" ]]; then
  n_workflows=$(find "${REPO_ROOT}/.github/workflows" -maxdepth 1 -type f \( -name '*.yml' -o -name '*.yaml' \) 2>/dev/null | wc -l | tr -d ' ')
  echo "- [GitHub Actions workflows] Met: ${n_workflows} 件" >> "${OSS_OUT}"
  if [[ -f "${REPO_ROOT}/.github/workflows/pr.yml" ]]; then
    echo "  - pr.yml: PR 単位で発火する workflow を確認" >> "${OSS_OUT}"
  fi
else
  echo "- [GitHub Actions workflows] Unmet: .github/workflows/ 不在" >> "${OSS_OUT}"
fi

# SAST (Scorecard 項目)
{
  echo
  echo "## SAST / 静的解析 (Scorecard 項目)"
  echo
} >> "${OSS_OUT}"
if [[ -f "${REPO_ROOT}/.github/workflows/_reusable-lint.yml" ]]; then
  echo "- [SAST] Met: _reusable-lint.yml で各言語 linter 実行" >> "${OSS_OUT}"
else
  echo "- [SAST] Unknown: 専用 reusable lint workflow 不在 (個別 workflow で対応している可能性)" >> "${OSS_OUT}"
fi
# 個別 linter ツール存在
if [[ -f "${REPO_ROOT}/.golangci.yml" || -f "${REPO_ROOT}/.golangci.yaml" ]]; then
  echo "  - golangci-lint config あり" >> "${OSS_OUT}"
fi
if find "${REPO_ROOT}" -maxdepth 3 -name 'clippy.toml' 2>/dev/null | grep -q . ; then
  echo "  - Rust clippy config あり" >> "${OSS_OUT}"
fi

# Token-Permissions (Scorecard 項目)
{
  echo
  echo "## Token-Permissions (Scorecard 項目)"
  echo
} >> "${OSS_OUT}"
if [[ -d "${REPO_ROOT}/.github/workflows" ]]; then
  n_with_permissions=$(grep -lr "^permissions:" "${REPO_ROOT}/.github/workflows" 2>/dev/null | wc -l | tr -d ' ')
  n_total=$(find "${REPO_ROOT}/.github/workflows" -type f \( -name '*.yml' -o -name '*.yaml' \) 2>/dev/null | wc -l | tr -d ' ')
  echo "- [permissions: 明示] ${n_with_permissions}/${n_total} workflow で permissions キー明示" >> "${OSS_OUT}"
  if [[ "${n_with_permissions}" -lt "${n_total}" ]]; then
    echo "  - 一部 workflow で permissions 未明示 (default = write、最小権限原則違反の可能性)" >> "${OSS_OUT}"
  fi
fi

# Pinned-Dependencies (Scorecard 項目)
{
  echo
  echo "## Pinned-Dependencies (Scorecard 項目)"
  echo
} >> "${OSS_OUT}"
[[ -f "${REPO_ROOT}/renovate.json" ]] && echo "- [Renovate config] Met: renovate.json (依存自動更新)" >> "${OSS_OUT}" || echo "- [Renovate config] Unmet" >> "${OSS_OUT}"
# lock files
lock_files=()
[[ -f "${REPO_ROOT}/go.sum" ]] && lock_files+=("go.sum")
find "${REPO_ROOT}" -maxdepth 4 -name 'Cargo.lock' 2>/dev/null | head -3 | while read -r f; do
  echo "  - ${f#${REPO_ROOT}/}" >> "${OSS_OUT}"
done
find "${REPO_ROOT}" -maxdepth 4 -name 'pnpm-lock.yaml' 2>/dev/null | head -3 | while read -r f; do
  echo "  - ${f#${REPO_ROOT}/}" >> "${OSS_OUT}"
done

# Fuzzing (Scorecard 項目)
{
  echo
  echo "## Fuzzing (Scorecard 項目)"
  echo
} >> "${OSS_OUT}"
if [[ -d "${REPO_ROOT}/tests/fuzz" ]]; then
  n_go_fuzz=$(find "${REPO_ROOT}/tests/fuzz" -name '*fuzz*test.go' 2>/dev/null | wc -l | tr -d ' ')
  n_rust_fuzz=$(find "${REPO_ROOT}/tests/fuzz" -path '*/rust/*' -name '*.rs' 2>/dev/null | wc -l | tr -d ' ')
  echo "- [Fuzz target] Met: tests/fuzz/ (Go ${n_go_fuzz} 件 / Rust ${n_rust_fuzz} 件)" >> "${OSS_OUT}"
else
  echo "- [Fuzz target] Unmet: tests/fuzz/ 不在" >> "${OSS_OUT}"
fi

# Signed-Releases / SBOM (Scorecard 項目)
{
  echo
  echo "## Signed-Releases / SBOM (Scorecard 項目)"
  echo
} >> "${OSS_OUT}"
if [[ -d "${REPO_ROOT}/ops/supply-chain" ]]; then
  n_sbom=0
  n_signatures=0
  [[ -d "${REPO_ROOT}/ops/supply-chain/sbom" ]] && n_sbom=$(find "${REPO_ROOT}/ops/supply-chain/sbom" -type f 2>/dev/null | wc -l | tr -d ' ')
  [[ -d "${REPO_ROOT}/ops/supply-chain/signatures" ]] && n_signatures=$(find "${REPO_ROOT}/ops/supply-chain/signatures" -type f 2>/dev/null | wc -l | tr -d ' ')
  echo "- [SBOM] Met: ops/supply-chain/sbom/ (${n_sbom} 件)" >> "${OSS_OUT}"
  echo "- [Signatures] Met: ops/supply-chain/signatures/ (${n_signatures} 件)" >> "${OSS_OUT}"
  [[ -d "${REPO_ROOT}/ops/supply-chain/keys" ]] && echo "- [Public Keys] Met: ops/supply-chain/keys/" >> "${OSS_OUT}"
else
  echo "- [SBOM / Signatures] Unmet: ops/supply-chain/ 不在" >> "${OSS_OUT}"
fi

# Binary-Artifacts (Scorecard 項目)
{
  echo
  echo "## Binary-Artifacts (Scorecard 項目)"
  echo
} >> "${OSS_OUT}"
n_binaries=$(cd "${REPO_ROOT}" && git ls-files 2>/dev/null | grep -E '\.(exe|dll|so|dylib|jar|class)$' | wc -l | tr -d ' ' || true)
n_binaries="${n_binaries:-0}"
if [[ "${n_binaries}" -eq 0 ]]; then
  echo "- [Commit 済バイナリ] Met: 0 件 (clean、git ls-files で .exe/.dll/.so/.dylib/.jar/.class なし)" >> "${OSS_OUT}"
else
  echo "- [Commit 済バイナリ] Unmet: ${n_binaries} 件 (要 .gitignore 整備)" >> "${OSS_OUT}"
fi

# Dangerous-Workflow (Scorecard 項目) — pull_request_target + checkout PR HEAD の組み合わせを検出
{
  echo
  echo "## Dangerous-Workflow (Scorecard 項目)"
  echo
} >> "${OSS_OUT}"
if [[ -d "${REPO_ROOT}/.github/workflows" ]]; then
  # 走査対象 workflow を find で一覧化（リテラル glob で grep が exit 2 する罠を回避）
  WF_LIST="${EVIDENCE_DIR}/.oss-workflows.tmp"
  find "${REPO_ROOT}/.github/workflows" -maxdepth 1 -type f \( -name '*.yml' -o -name '*.yaml' \) > "${WF_LIST}" 2>/dev/null || true
  n_prt=0
  n_dangerous=0
  while IFS= read -r wf; do
    [[ -f "${wf}" ]] || continue
    if grep -qE 'pull_request_target' "${wf}" 2>/dev/null; then
      n_prt=$((n_prt + 1))
      if grep -qE 'actions/checkout' "${wf}" 2>/dev/null && \
         grep -qE 'github\.event\.pull_request\.head\.(ref|sha)' "${wf}" 2>/dev/null; then
        n_dangerous=$((n_dangerous + 1))
        echo "  - dangerous combo in: $(basename "${wf}")" >> "${OSS_OUT}"
      fi
    fi
  done < "${WF_LIST}"
  rm -f "${WF_LIST}"
  if [[ "${n_dangerous}" -eq 0 ]]; then
    echo "- [Dangerous-Workflow] Met: pull_request_target + PR HEAD checkout の危険な組み合わせ 0 件 (全 ${n_prt} 件中)" >> "${OSS_OUT}"
  else
    echo "- [Dangerous-Workflow] Unmet: ${n_dangerous} 件 (要 review)" >> "${OSS_OUT}"
  fi
fi

# CII Best Practices Passing 17 項目 — ローカルで判定可能なものを採点
{
  echo
  echo "## CII / OpenSSF Best Practices Badge (Passing 17 項目、ローカル判定可能分)"
  echo
} >> "${OSS_OUT}"

cii_check() {
  local label="$1"
  local met="$2"
  local note="$3"
  if [[ "${met}" == "true" ]]; then
    echo "- [CII: ${label}] Met: ${note}" >> "${OSS_OUT}"
  elif [[ "${met}" == "manual" ]]; then
    echo "- [CII: ${label}] Manual-Required: ${note}" >> "${OSS_OUT}"
  else
    echo "- [CII: ${label}] Unmet: ${note}" >> "${OSS_OUT}"
  fi
}

# Basics 領域
cii_check "project_url" "$([[ -f "${REPO_ROOT}/README.md" ]] && echo true || echo false)" "README.md = project ホーム"
cii_check "vulnerability_report" "$([[ -f "${REPO_ROOT}/SECURITY.md" ]] && echo true || echo false)" "SECURITY.md = vulnerability 報告経路"
cii_check "floss_license" "$([[ -f "${REPO_ROOT}/LICENSE" ]] && echo true || echo false)" "LICENSE 存在 (FLOSS license)"

# Change Control 領域
public_vcs="false"
if git -C "${REPO_ROOT}" remote -v 2>/dev/null | grep -qE 'github\.com|gitlab\.com|bitbucket\.org'; then
  public_vcs="true"
fi
cii_check "public_version_control" "${public_vcs}" "git remote が public ホスティング先を指している"
n_tags=$(git -C "${REPO_ROOT}" tag 2>/dev/null | wc -l | tr -d ' ')
cii_check "unique_version_numbering" "$([[ "${n_tags}" -gt 0 ]] && echo true || echo manual)" "git tag ${n_tags} 件 (semver 準拠は別途確認)"
cii_check "release_notes" "$([[ -f "${REPO_ROOT}/CHANGELOG.md" ]] && echo true || echo manual)" "CHANGELOG.md の有無 ($([[ -f "${REPO_ROOT}/CHANGELOG.md" ]] && echo 存在 || echo 不在))"

# Reporting 領域
cii_check "bug_reporting_process" "$([[ -d "${REPO_ROOT}/.github/ISSUE_TEMPLATE" ]] && echo true || echo false)" ".github/ISSUE_TEMPLATE/ で報告経路"
cii_check "vulnerability_response" "$([[ -f "${REPO_ROOT}/SECURITY.md" ]] && echo true || echo manual)" "SECURITY.md (応答 SLA は別途記述要)"

# Quality 領域
build_evidence="manual"
if [[ -f "${REPO_ROOT}/Makefile" ]] || find "${REPO_ROOT}" -maxdepth 2 -name 'Cargo.toml' -o -name 'go.mod' -o -name 'package.json' 2>/dev/null | grep -q .; then
  build_evidence="true"
fi
cii_check "working_build_system" "${build_evidence}" "Makefile / Cargo.toml / go.mod / package.json のいずれか"
test_evidence="false"
if [[ -d "${REPO_ROOT}/tests" ]] || find "${REPO_ROOT}" -maxdepth 4 -name '*_test.go' -o -name '*_test.rs' 2>/dev/null | head -1 | grep -q .; then
  test_evidence="true"
fi
cii_check "automated_test_suite" "${test_evidence}" "tests/ ディレクトリ or *_test.{go,rs} 存在"
cii_check "test_added_for_changes" "manual" "PR レビュープロセスで test 追加を要求 (PR template で確認)"

# Security 領域
cii_check "use_basic_good_cryptographic_practices" "manual" "暗号アルゴリズム選定の妥当性は人間判定 (ADR-CRYPTO-* 系参照)"
cii_check "secured_delivery" "$([[ -d "${REPO_ROOT}/ops/supply-chain" ]] && echo true || echo false)" "ops/supply-chain/ (sbom + signatures + keys)"
cii_check "publicly_known_vulnerabilities_fixed" "manual" "dependabot / renovate 自動更新 + lock file 監査"

# Analysis 領域
cii_check "static_analysis" "$([[ -f "${REPO_ROOT}/.github/workflows/_reusable-lint.yml" ]] && echo true || echo manual)" "_reusable-lint.yml = SAST workflow"

# 集計
n_cii_met=$(grep -c '^- \[CII: .*\] Met:' "${OSS_OUT}" 2>/dev/null || true)
n_cii_unmet=$(grep -c '^- \[CII: .*\] Unmet:' "${OSS_OUT}" 2>/dev/null || true)
n_cii_manual=$(grep -c '^- \[CII: .*\] Manual-Required:' "${OSS_OUT}" 2>/dev/null || true)
n_cii_met="${n_cii_met:-0}"
n_cii_unmet="${n_cii_unmet:-0}"
n_cii_manual="${n_cii_manual:-0}"
{
  echo
  echo "### CII Best Practices Passing 集計"
  echo "- Met (機械判定): ${n_cii_met} / 17"
  echo "- Unmet: ${n_cii_unmet} / 17"
  echo "- Manual-Required (人間判定): ${n_cii_manual} / 17"
  echo "- 外部 bestpractices.dev での最終登録は別途実施 (URL 自己申告制)"
} >> "${OSS_OUT}"

# Branch-Protection / Code-Review / Vulnerabilities (public repo + GitHub API 必須)
{
  echo
  echo "## public repo + GitHub API 必須項目 (Unknown)"
  echo
  echo "- [Branch-Protection] Unknown: GitHub Settings の API、public 化後に scorecard-cli で確認"
  echo "- [Code-Review] Unknown: PR 履歴の分析、public 化後に scorecard-cli で確認"
  echo "- [Vulnerabilities] Unknown: dependabot alert、public 化後に確認"
  echo "- [Webhooks] N/A: public 化前のため対象外"
} >> "${OSS_OUT}"

# OSSF Scorecard CLI (機械採点ツール)
{
  echo
  echo "## OSSF Scorecard CLI"
  echo
} >> "${OSS_OUT}"
if command -v scorecard >/dev/null 2>&1 || command -v scorecard-cli >/dev/null 2>&1; then
  echo "- scorecard CLI 検出: 実行は別途必要 (公開 repo URL を引数で渡す)" >> "${OSS_OUT}"
else
  echo "- status: scorecard_not_installed" >> "${OSS_OUT}"
  echo "- note: scorecard-cli を導入後に手動実行 / GitHub Action で自動採点" >> "${OSS_OUT}"
fi

# Maintained (Scorecard 項目) — git history からメンテ頻度
{
  echo
  echo "## Maintained (Scorecard 項目) - git log 直近 30 日 / 90 日"
  echo
  cd "${REPO_ROOT}" 2>/dev/null && {
    commits_30d=$(safe_count "git log --since='30 days ago' --oneline 2>/dev/null | wc -l | tr -d ' '")
    commits_90d=$(safe_count "git log --since='90 days ago' --oneline 2>/dev/null | wc -l | tr -d ' '")
    contributors_all=$(safe_count "git log --format='%ae' 2>/dev/null | sort -u | wc -l | tr -d ' '")
    echo "- commits_last_30_days: ${commits_30d}"
    echo "- commits_last_90_days: ${commits_90d} (Scorecard Maintained 閾値)"
    echo "- unique_contributors_all_time: ${contributors_all}"
    if [[ "${commits_90d}" -gt 0 ]]; then
      echo "- [Maintained] Met (90 日以内に commit あり)"
    else
      echo "- [Maintained] Unmet (90 日以内に commit なし)"
    fi
  } || echo "- git log 取得不可"
} >> "${OSS_OUT}"

# 集計サマリ
{
  echo
  echo "## 集計サマリ"
  echo
  # 「Met:」「Unmet:」「Unknown」を含む説明行（# 判定値: 等）を除外し、項目行のみカウント
  n_met=$(grep -c '^- \[.*\] Met:' "${OSS_OUT}" 2>/dev/null || true)
  n_unmet=$(grep -c '^- \[.*\] Unmet:' "${OSS_OUT}" 2>/dev/null || true)
  n_unknown=$(grep -c '^- \[.*\] Unknown' "${OSS_OUT}" 2>/dev/null || true)
  n_met="${n_met:-0}"; n_unmet="${n_unmet:-0}"; n_unknown="${n_unknown:-0}"
  echo "- Met: ${n_met} 件"
  echo "- Unmet: ${n_unmet} 件"
  echo "- Unknown (public repo + scorecard-cli 必須): ${n_unknown} 件"
  echo
  echo "判定総合は人間が docs/AUDIT.md D 軸で行うこと (audit-protocol skill 規約)。"
  echo "Best Practices Badge の Passing 17 項目自己採点は外部サイト https://www.bestpractices.dev/ で repo URL を入力して実施。"
} >> "${OSS_OUT}"

echo "=== oss 軸 ==="
cat "${OSS_OUT}"
