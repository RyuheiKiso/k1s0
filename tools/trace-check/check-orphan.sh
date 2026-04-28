#!/usr/bin/env bash
#
# tools/trace-check/check-orphan.sh — ADR/DS-SW-COMP/NFR マトリクス孤立 ID 検出
#
# 設計: docs/05_実装/99_索引/50_整合性CI/01_整合性CI設計.md
# 関連 IMP-ID: IMP-TRACE-CI-013 / IMP-TRACE-CI-010
# 責務:
#   台帳に採番済として登録されているすべての非 POL IMP-* ID について、
#   3 マトリクス（ADR対応 / DS-SW-COMP-IMP対応 / NFR-IMP対応）の少なくとも
#   1 つで参照されているかを確認する。
#   3 マトリクス全てで参照ゼロの ID は「孤立 ID」として warning を出力する。
#   --strict モードでは孤立 ID を FAIL に格上げする（IMP-TRACE-CI-018 月次 cron 用）。
#
# Usage:
#   tools/trace-check/check-orphan.sh [--strict] [--help]
#
# Exit code:
#   0 = pass（孤立 ID なし、または warning のみ）
#   1 = fail（--strict 時の孤立 ID 検出）
#   2 = setup error

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
DOCS_IDX="${REPO_ROOT}/docs/05_実装/99_索引"
LEDGER_FILE="${DOCS_IDX}/00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md"
MATRIX_ADR="${DOCS_IDX}/10_ADR対応表/01_ADR-IMP対応マトリクス.md"
MATRIX_DS="${DOCS_IDX}/20_DS-SW-COMP対応表/01_DS-SW-COMP-IMP対応マトリクス.md"
MATRIX_NFR="${DOCS_IDX}/30_NFR対応表/01_NFR-IMP対応マトリクス.md"
STRICT=0
FAIL=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --strict) STRICT=1; shift ;;
    -h|--help)
      sed -n '3,22p' "$0" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *)
      echo "[error] 未知のオプション: $1" >&2
      exit 2
      ;;
  esac
done

# ──────────────────────────────────────────────────────────
# 依存チェック
# ──────────────────────────────────────────────────────────
for cmd in grep sort; do
  if ! command -v "${cmd}" >/dev/null 2>&1; then
    echo "[setup-error] ${cmd} が見つかりません" >&2
    exit 2
  fi
done

for f in "${LEDGER_FILE}" "${MATRIX_ADR}" "${MATRIX_DS}" "${MATRIX_NFR}"; do
  if [[ ! -f "${f}" ]]; then
    echo "[setup-error] ファイルが見つかりません: ${f}" >&2
    exit 2
  fi
done

# ──────────────────────────────────────────────────────────
# 台帳から非 POL 採番済 ID を抽出
# ──────────────────────────────────────────────────────────
IMPL_IDS_FILE="$(mktemp)"
trap 'rm -f "${IMPL_IDS_FILE}"' EXIT

# POL ID（IMP-XXX-POL-NNN）を除いた詳細 ID 行
grep -oE 'IMP-[A-Z]+-(?!POL)[A-Z]+-[0-9]{3}' "${LEDGER_FILE}" 2>/dev/null \
  | sort -u > "${IMPL_IDS_FILE}" \
  || grep -oE 'IMP-[A-Z]+-[A-Z]+-[0-9]{3}' "${LEDGER_FILE}" \
      | grep -v '\-POL-' \
      | sort -u > "${IMPL_IDS_FILE}"

impl_count="$(wc -l < "${IMPL_IDS_FILE}" | tr -d ' ')"
echo "[info] 台帳内 非POL 採番済 ID 総数: ${impl_count}"

# ──────────────────────────────────────────────────────────
# 3 マトリクスに登場する ID を集約
# ──────────────────────────────────────────────────────────
MATRIX_IDS_FILE="$(mktemp)"
trap 'rm -f "${IMPL_IDS_FILE}" "${MATRIX_IDS_FILE}"' EXIT

{
  grep -oE 'IMP-[A-Z]+-[A-Z]+-[0-9]{3}' "${MATRIX_ADR}" 2>/dev/null || true
  grep -oE 'IMP-[A-Z]+-[A-Z]+-[0-9]{3}' "${MATRIX_DS}"  2>/dev/null || true
  grep -oE 'IMP-[A-Z]+-[A-Z]+-[0-9]{3}' "${MATRIX_NFR}" 2>/dev/null || true
} | sort -u > "${MATRIX_IDS_FILE}"

matrix_count="$(wc -l < "${MATRIX_IDS_FILE}" | tr -d ' ')"
echo "[info] 3 マトリクスに登場する ID 総数: ${matrix_count}"

# ──────────────────────────────────────────────────────────
# 孤立 ID 検出（台帳にあるがどのマトリクスにも参照されない）
# ──────────────────────────────────────────────────────────
ORPHAN_IDS="$(comm -23 "${IMPL_IDS_FILE}" "${MATRIX_IDS_FILE}")"
orphan_count=0

echo ""
echo "=== 孤立 ID 一覧（3 マトリクス全てで参照なし） ==="
if [[ -n "${ORPHAN_IDS}" ]]; then
  orphan_count="$(echo "${ORPHAN_IDS}" | wc -l | tr -d ' ')"
  echo "${ORPHAN_IDS}" | while read -r id; do
    echo "  [warn] ${id}: ADR/DS-SW-COMP/NFR マトリクスから参照されていない"
  done
  if [[ "${STRICT}" -eq 1 ]]; then
    FAIL=1
  fi
else
  echo "  なし（正常）"
fi

# ──────────────────────────────────────────────────────────
# 結果サマリ
# ──────────────────────────────────────────────────────────
echo ""
if [[ "${FAIL}" -eq 1 ]]; then
  echo "[result] FAIL (strict) — 孤立 ID が ${orphan_count} 件あります"
  echo "         修正候補: 各マトリクスに参照を追加するか ADR/NFR/DS-SW-COMP を紐付けること"
  exit 1
else
  echo "[result] PASS — 孤立 ID チェック完了"
  if [[ "${orphan_count}" -gt 0 ]]; then
    echo "         warn: 孤立 ID ${orphan_count} 件（30 日後の月次 cron で Sev3 通知対象）"
  fi
  exit 0
fi
