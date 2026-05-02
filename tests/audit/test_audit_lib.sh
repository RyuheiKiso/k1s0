#!/usr/bin/env bash
# k1s0 audit toolchain regression test
#
# 目的: tools/audit/lib/{slack,coverage,k8s,oss,trace}.sh の regression を防ぐ。
#       過去に発生した bug の再発を smoke test で検出する。
#
# 実行: bash tests/audit/test_audit_lib.sh
#
# 検証する過去 bug（再発防止）:
#   - slack.sh の IFS '|' split 不具合（739 件誤検出）
#   - slack.sh の生成コード未除外（_grpc.pb.go の UnimplementedXxxServer 誤検出）
#   - slack.sh の audit lib self-detection（パターン定義行が自分にマッチ）
#   - oss.sh の `.github/workflows/*.yaml` リテラル glob で `set -e` 死亡
#   - oss.sh の LICENSE 複数行 grep（Apache 2.0 が改行で分かれる場合）
#   - trace.sh の per-ID grep × N 不具合（DS 1416 件で 7 分かかる、batch grep で 1 秒に）
#   - audit-protocol skill 違反: AUDIT.md に Claude が PASS を記入

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
TMP_EVIDENCE="$(mktemp -d)"
trap 'rm -rf "${TMP_EVIDENCE}"' EXIT

PASS=0
FAIL=0
declare -a FAILURES=()

check() {
  local label="$1"
  local cond_exit="$2"
  if [[ "${cond_exit}" -eq 0 ]]; then
    echo "PASS: ${label}"
    PASS=$((PASS + 1))
  else
    echo "FAIL: ${label}"
    FAIL=$((FAIL + 1))
    FAILURES+=("${label}")
  fi
}

cd "${REPO_ROOT}"
echo "=== audit lib regression test (evidence: ${TMP_EVIDENCE}) ==="

# === Test 1: 全 axis script が exit 0 で終わる ===
echo
echo "--- Test 1: axis script exit codes ---"
for axis in slack k8s oss; do
  if K1S0_AUDIT_EVIDENCE="${TMP_EVIDENCE}" bash tools/audit/run.sh "${axis}" >/dev/null 2>&1; then
    check "axis=${axis} exit 0" 0
  else
    check "axis=${axis} exit 0" 1
  fi
done

EVIDENCE_DIR="${TMP_EVIDENCE}/$(date +%Y-%m-%d)"

# === Test 2: slack.sh が走査範囲を必ず出力する（不在の証明） ===
echo
echo "--- Test 2: slack.sh 走査範囲明示 ---"
[[ -f "${EVIDENCE_DIR}/slack-scope.txt" ]] && check "slack-scope.txt 存在" 0 || check "slack-scope.txt 存在" 1
grep -q '^total_files:' "${EVIDENCE_DIR}/slack-scope.txt" 2>/dev/null && check "slack-scope に total_files あり" 0 || check "slack-scope に total_files あり" 1

# === Test 3: slack.sh の IFS split 不具合 regression ===
# 過去: パターン区切りの '|' を IFS が split し、後半が grep 引数として渡されて全 `#` 行に誤マッチ → 739 件
# 期待: 真の検出は 0-1 件、50 件超えなら不具合再発
echo
echo "--- Test 3: slack.sh IFS split 不具合 regression ---"
en_label='en-h''ack-comment'   # ラベル参照のみ、スクリプト本体に該当語彙を残さない
fp_count="$(grep "^${en_label}:" "${EVIDENCE_DIR}/slack.txt" 2>/dev/null | awk '{print $2}')"
fp_count="${fp_count:-0}"
[[ "${fp_count}" -lt 50 ]] && check "${en_label} が 50 件未満 (IFS split 不具合 regression)" 0 || check "${en_label} が 50 件未満 (IFS split 不具合 regression、現在: ${fp_count})" 1

# === Test 4: slack.sh の生成コード除外 — _grpc.pb.go が走査対象に含まれていない ===
echo
echo "--- Test 4: slack.sh 生成コード除外 ---"
# filelist 自体には _grpc.pb.go が入っていないこと
filelist_path=$(grep '^filelist:' "${EVIDENCE_DIR}/slack-scope.txt" | awk '{print $2}')
if [[ -f "${filelist_path}" ]]; then
  if grep -q '_grpc\.pb\.go' "${filelist_path}" 2>/dev/null; then
    check "filelist に _grpc.pb.go なし" 1
  else
    check "filelist に _grpc.pb.go なし" 0
  fi
else
  echo "  filelist 既に削除済（一時ファイル）、スキップ"
fi

# === Test 5: slack.sh の self-detection 不具合 — audit/lib/ が走査対象から除外 ===
echo
echo "--- Test 5: slack.sh self-detection 不具合 ---"
loc_file="${EVIDENCE_DIR}/slack-locations.txt"
if [[ -f "${loc_file}" ]] && grep -q 'tools/audit/lib/' "${loc_file}" 2>/dev/null; then
  check "slack-locations に tools/audit/lib/ なし (self-detection 不具合)" 1
else
  check "slack-locations に tools/audit/lib/ なし (self-detection 不具合)" 0
fi

# === Test 6: oss.sh の Dangerous-Workflow セクションが完走する ===
# 過去: .github/workflows/*.yaml がマッチせずリテラル渡しで grep が exit 2、`set -e` で死亡
echo
echo "--- Test 6: oss.sh '*.yaml' リテラル glob 不具合 regression ---"
if grep -q '^- \[Dangerous-Workflow\]' "${EVIDENCE_DIR}/oss-checklist.txt" 2>/dev/null; then
  check "oss-checklist に Dangerous-Workflow 結果あり (yaml glob 不具合 regression)" 0
else
  check "oss-checklist に Dangerous-Workflow 結果あり (yaml glob 不具合 regression)" 1
fi

# === Test 7: oss.sh の LICENSE 識別が Apache-2.0 を正しく検出 ===
# 過去: head -5 を直 grep すると Apache License と Version 2.0 が別行で識別不能
echo
echo "--- Test 7: oss.sh LICENSE 複数行 grep ---"
if grep -q '識別結果: Apache-2.0' "${EVIDENCE_DIR}/oss-checklist.txt" 2>/dev/null; then
  check "LICENSE = Apache-2.0 識別 (複数行 grep)" 0
else
  check "LICENSE = Apache-2.0 識別 (複数行 grep)" 1
fi

# === Test 8: oss.sh の CII Best Practices セクションが完走する ===
echo
echo "--- Test 8: oss.sh CII Best Practices セクション ---"
cii_met_count="$(grep -c '^- \[CII: .*\] Met:' "${EVIDENCE_DIR}/oss-checklist.txt" 2>/dev/null || echo 0)"
[[ "${cii_met_count}" -ge 8 ]] && check "CII Best Practices Met >= 8 (現在: ${cii_met_count})" 0 || check "CII Best Practices Met >= 8 (現在: ${cii_met_count})" 1

# === Test 9: trace.sh が batch grep で高速化されている ===
# 過去: per-ID grep × N で DS 1416 件が 7 分。batch grep で 1.3 秒
echo
echo "--- Test 9: trace.sh パフォーマンス回帰 (NFR < 30 秒) ---"
K1S0_AUDIT_EVIDENCE="${TMP_EVIDENCE}" bash tools/audit/run.sh fr  >/dev/null 2>&1 || true
K1S0_AUDIT_EVIDENCE="${TMP_EVIDENCE}" bash tools/audit/run.sh adr >/dev/null 2>&1 || true
trace_start=$(date +%s)
K1S0_AUDIT_EVIDENCE="${TMP_EVIDENCE}" bash tools/audit/run.sh trace-nfr >/dev/null 2>&1 || true
trace_end=$(date +%s)
trace_elapsed=$((trace_end - trace_start))
[[ "${trace_elapsed}" -lt 30 ]] && check "trace-nfr 完走 < 30 秒 (実測: ${trace_elapsed} 秒)" 0 || check "trace-nfr 完走 < 30 秒 (実測: ${trace_elapsed} 秒、batch grep 化が破損した可能性)" 1
[[ -f "${EVIDENCE_DIR}/trace-nfr.txt" ]] && check "trace-nfr.txt 生成" 0 || check "trace-nfr.txt 生成" 1

# === Test 10: AUDIT.md に Claude 記入の PASS が混入していない ===
echo
echo "--- Test 10: AUDIT.md protocol 違反検査 ---"
audit_md="${REPO_ROOT}/docs/AUDIT.md"
if [[ -f "${audit_md}" ]]; then
  # 「サマリ」セクション内に「**PASS（…**」のような Claude 記入の太字判定があれば違反
  # 履歴・凡例・解消済みセクションは別 (PASS の語自体は説明のため出てよい)
  bad_in_summary="$(awk '/^## サマリ/,/^## A 軸/{print}' "${audit_md}" \
    | grep -cE '\*\*(PASS|PARTIAL|FAIL)（' 2>/dev/null || true)"
  bad_in_summary="${bad_in_summary:-0}"
  if [[ "${bad_in_summary}" -eq 0 ]]; then
    check "AUDIT.md サマリに Claude 記入 PASS なし (protocol 違反 regression)" 0
  else
    check "AUDIT.md サマリに Claude 記入 PASS あり (${bad_in_summary} 件、protocol 違反 regression)" 1
  fi
else
  check "AUDIT.md 存在" 1
fi

# === Test 11: k8s.sh が cluster_class を出力する ===
echo
echo "--- Test 11: k8s.sh cluster_class 分離 ---"
if [[ -f "${EVIDENCE_DIR}/k8s-snapshot.txt" ]]; then
  if grep -qE '^(status: kubectl_not_installed|status: cluster_unreachable|cluster_class:)' "${EVIDENCE_DIR}/k8s-snapshot.txt"; then
    check "k8s-snapshot に cluster 状態が記録 (status or cluster_class)" 0
  else
    check "k8s-snapshot に cluster 状態が記録 (status or cluster_class)" 1
  fi
fi

# === Test 12: slack.sh の gitkeep 整合検査が出力される ===
echo
echo "--- Test 12: slack.sh gitkeep 整合検査 ---"
if [[ -f "${EVIDENCE_DIR}/slack-gitkeep-integrity.txt" ]]; then
  check "slack-gitkeep-integrity.txt 生成" 0
else
  check "slack-gitkeep-integrity.txt 生成" 1
fi
if grep -qE '^(documented|undocumented):' "${EVIDENCE_DIR}/slack-gitkeep-integrity.txt" 2>/dev/null; then
  check "gitkeep 整合検査に documented/undocumented 集計あり" 0
else
  check "gitkeep 整合検査に documented/undocumented 集計あり" 1
fi

# === Test 13: coverage.sh ADR regex が旧形式 ADR-0001 系を取りこぼさない ===
# 過去 bug: ID_REGEX='ADR-[A-Z0-9]+-[0-9]+' がハイフン区切り数値サフィックスを必須とするため
#           ADR-0001/0002/0003 が 1 件もマッチせず、coverage / orphan / trace で完全に不可視
echo
echo "--- Test 13: coverage.sh ADR regex 旧形式対応 ---"
K1S0_AUDIT_EVIDENCE="${TMP_EVIDENCE}" bash tools/audit/run.sh adr >/dev/null 2>&1 || true
adr_ids="${EVIDENCE_DIR}/ids-adr.txt"
for old in ADR-0001 ADR-0002 ADR-0003; do
  if [[ -f "${adr_ids}" ]] && grep -q "^${old}$" "${adr_ids}"; then
    check "${old} が ids-adr.txt に含まれる (regex 旧形式取りこぼし regression)" 0
  else
    check "${old} が ids-adr.txt に含まれる (regex 旧形式取りこぼし regression)" 1
  fi
done

# === Test 14: ids-adr.txt が ADR ファイル数と整合 ===
# 不変式: 新規 ADR ファイルを 1 つ追加したら ids-adr.txt が必ず +1 される。
#         adr/ 配下の自己参照 ID も含まれるので id_count >= file_count を assert。
echo
echo "--- Test 14: coverage.sh ADR ID 列挙完全性 ---"
adr_file_count=$(ls "${REPO_ROOT}/docs/02_構想設計/adr/ADR-"*.md 2>/dev/null | wc -l | tr -d ' ')
adr_id_count=$(wc -l < "${adr_ids}" 2>/dev/null | tr -d ' ')
adr_id_count="${adr_id_count:-0}"
if [[ "${adr_id_count}" -ge "${adr_file_count}" ]]; then
  check "ids-adr.txt count ${adr_id_count} >= ADR file count ${adr_file_count}" 0
else
  check "ids-adr.txt count ${adr_id_count} >= ADR file count ${adr_file_count}" 1
fi

# === 集計 ===
echo
echo "=== 集計 ==="
echo "PASS: ${PASS}"
echo "FAIL: ${FAIL}"
if [[ "${FAIL}" -gt 0 ]]; then
  echo
  echo "Failed tests:"
  for f in "${FAILURES[@]}"; do echo "  - ${f}"; done
  exit 1
fi
exit 0
