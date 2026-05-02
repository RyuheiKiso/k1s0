#!/usr/bin/env bash
# k1s0 監査エントリスクリプト
#
# 判定基準の正典: docs/00_format/audit_criteria.md
# 監査方法論 skill: .claude/skills/audit-protocol/SKILL.md
#
# 使い方:
#   tools/audit/run.sh slack         # B 軸: 手抜き検出
#   tools/audit/run.sh fr            # A 軸: FR ID 網羅
#   tools/audit/run.sh nfr           # A 軸: NFR ID 網羅
#   tools/audit/run.sh ds            # A 軸: DS ID 網羅
#   tools/audit/run.sh imp           # A 軸: IMP ID 網羅
#   tools/audit/run.sh adr           # A 軸: ADR 網羅 + orphan 検出
#   tools/audit/run.sh k8s           # C 軸: k8s 実機状態スナップショット
#   tools/audit/run.sh oss           # D 軸: OSS 完成度
#   tools/audit/run.sh all           # 全軸
#
# 環境変数:
#   K1S0_AUDIT_DATE     証跡保存先の日付（既定: 今日）
#   K1S0_AUDIT_EVIDENCE 証跡保存先ディレクトリ（既定: .claude/audit-evidence）
#
# 出力: 標準出力にサマリ、証跡ファイルは ${K1S0_AUDIT_EVIDENCE}/<date>/<axis>.txt 等
#
# 設計原則:
#   - PASS / FAIL を本 script は判定しない（証跡を集めて並べる）
#   - 全コマンドの実行時刻 / 走査範囲 / コマンド本文を meta.txt に記録
#   - 不在の証明として、走査ファイル数を必ず併記する

set -euo pipefail

# リポジトリルートを cd 不要で計算
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
LIB_DIR="${SCRIPT_DIR}/lib"

DATE="${K1S0_AUDIT_DATE:-$(date +%Y-%m-%d)}"
EVIDENCE_BASE="${K1S0_AUDIT_EVIDENCE:-${REPO_ROOT}/.claude/audit-evidence}"
EVIDENCE_DIR="${EVIDENCE_BASE}/${DATE}"
mkdir -p "${EVIDENCE_DIR}"

# meta.txt: 実行コンテキストの記録（再現性確保）
write_meta() {
  local axis="$1"
  {
    echo "# k1s0 audit meta — ${axis}"
    echo "executed_at: $(date -Iseconds)"
    echo "commit: $(git -C "${REPO_ROOT}" rev-parse HEAD 2>/dev/null || echo 'NOT_A_GIT_REPO')"
    echo "branch: $(git -C "${REPO_ROOT}" rev-parse --abbrev-ref HEAD 2>/dev/null || echo 'NOT_A_GIT_REPO')"
    echo "uname: $(uname -a)"
    echo "axis: ${axis}"
    echo "evidence_dir: ${EVIDENCE_DIR}"
  } >> "${EVIDENCE_DIR}/meta.txt"
}

run_axis() {
  local axis="$1"
  case "${axis}" in
    slack)
      write_meta slack
      bash "${LIB_DIR}/slack.sh" "${REPO_ROOT}" "${EVIDENCE_DIR}"
      ;;
    fr|nfr|ds|imp|adr)
      write_meta "${axis}"
      bash "${LIB_DIR}/coverage.sh" "${REPO_ROOT}" "${EVIDENCE_DIR}" "${axis}"
      ;;
    k8s)
      write_meta k8s
      bash "${LIB_DIR}/k8s.sh" "${REPO_ROOT}" "${EVIDENCE_DIR}"
      ;;
    oss)
      write_meta oss
      bash "${LIB_DIR}/oss.sh" "${REPO_ROOT}" "${EVIDENCE_DIR}"
      ;;
    all)
      for a in slack fr nfr ds imp adr k8s oss; do
        echo "=== axis: ${a} ==="
        run_axis "${a}"
      done
      ;;
    *)
      echo "unknown axis: ${axis}" >&2
      echo "usage: $0 {slack|fr|nfr|ds|imp|adr|k8s|oss|all}" >&2
      return 2
      ;;
  esac
}

if [[ $# -lt 1 ]]; then
  echo "usage: $0 {slack|fr|nfr|ds|imp|adr|k8s|oss|all}" >&2
  exit 2
fi

run_axis "$1"
echo
echo "evidence saved to: ${EVIDENCE_DIR}"
