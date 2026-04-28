#!/usr/bin/env bash
#
# tools/trace-check/check-duplicate.sh — 同一 IMP-* ID の重複採番検出
#
# 設計: docs/05_実装/99_索引/50_整合性CI/01_整合性CI設計.md
# 関連 IMP-ID: IMP-TRACE-CI-014 / IMP-TRACE-CI-010
# 責務:
#   台帳（01_IMP-ID台帳_全12接頭辞.md）および各章 90_対応索引で
#   同一 IMP-* ID が 2 回以上出現するケースを検出する。
#   重複は別 PR で異なる担当者が同じ ID を使用した「レース」でも発生する。
#   CI で即時 FAIL とし ID 衝突を防ぐ。
#
# Usage:
#   tools/trace-check/check-duplicate.sh [--strict] [--help]
#
# Exit code:
#   0 = pass（重複なし）
#   1 = fail（重複 ID あり）
#   2 = setup error

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
DOCS_IMPL="${REPO_ROOT}/docs/05_実装"
LEDGER_FILE="${DOCS_IMPL}/99_索引/00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md"
STRICT=0
FAIL=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --strict) STRICT=1; shift ;;
    -h|--help)
      sed -n '3,21p' "$0" | sed 's/^# \{0,1\}//'
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
for cmd in grep sort uniq find; do
  if ! command -v "${cmd}" >/dev/null 2>&1; then
    echo "[setup-error] ${cmd} が見つかりません" >&2
    exit 2
  fi
done

if [[ ! -f "${LEDGER_FILE}" ]]; then
  echo "[setup-error] 台帳ファイルが見つかりません: ${LEDGER_FILE}" >&2
  exit 2
fi

# ──────────────────────────────────────────────────────────
# 台帳内の「定義行」重複 ID を検出
#
# 範囲記法（IMP-XXX-YYY-NNN〜MMM）や本文中での参照（prose）は除外し、
# `^| IMP-XXX-YYY-NNN ... |` 形式の定義行（または範囲記法 `^| IMP-XXX-YYY-NNN〜MMM`）
# が同じ単一 ID で 2 回以上書かれているケースのみ FAIL とする。
# ──────────────────────────────────────────────────────────
echo "=== 台帳内の「定義行」重複 IMP-* ID 検出 ==="
LEDGER_DUPS="$(
  grep -oE '^\|[[:space:]]*IMP-[A-Z]+-[A-Z]+-[0-9]{3}[[:space:]]*\|' "${LEDGER_FILE}" \
    | sed -E 's|^\|[[:space:]]*||; s|[[:space:]]*\|.*$||' \
    | sort \
    | uniq -d
)"

if [[ -n "${LEDGER_DUPS}" ]]; then
  echo "${LEDGER_DUPS}" | while read -r id; do
    count="$(grep -cE "^\\|[[:space:]]*${id}[[:space:]]*\\|" "${LEDGER_FILE}")"
    echo "  [FAIL] ${id}: 台帳内に ${count} 回定義行で出現（重複）"
  done
  FAIL=1
else
  echo "  なし（正常）"
fi

# ──────────────────────────────────────────────────────────
# 全 docs/05_実装/ 配下の重複 *定義* ID を検出
#
# 「定義行」とは Markdown テーブルの先頭セル `| IMP-XXX-YYY-NNN |` の形式に
# 限定する（左マージン後に `|` で囲まれた ID 単独セル）。これにより本文中の
# 言及・引用は除外し、別ファイル間で同じ ID を別の意味で **定義** している
# ケースのみを FAIL とする。台帳と 90_対応索引は同 ID を別箇所で定義する
# ことが正常運用なので、ここでは敢えて広く拾わない。
# ──────────────────────────────────────────────────────────
echo ""
echo "=== 05_実装/ 配下の「定義行」重複 IMP-* ID 検出 ==="
ALL_DUPS="$(
  find "${DOCS_IMPL}" -type f -name "*.md" \
    -exec grep -oHE '^\|[[:space:]]*IMP-[A-Z]+-[A-Z]+-[0-9]{3}[[:space:]]*\|' {} + 2>/dev/null \
    | sed -E 's|^.*:\|[[:space:]]*||; s|[[:space:]]*\|.*$||' \
    | sort \
    | uniq -d
)"

if [[ -n "${ALL_DUPS}" ]]; then
  echo "${ALL_DUPS}" | while read -r id; do
    echo "  [FAIL] ${id}: docs/05_実装/ 配下の複数ファイルで重複「定義」"
    # 定義箇所のみを表示（参照箇所は除外）
    find "${DOCS_IMPL}" -type f -name "*.md" \
      -exec grep -l -E "^\\|[[:space:]]*${id}[[:space:]]*\\|" {} + 2>/dev/null \
      | sed 's|'"${REPO_ROOT}"'/||' \
      | while read -r f; do
          echo "         → ${f}"
        done
  done
  FAIL=1
else
  echo "  なし（正常）"
fi

# ──────────────────────────────────────────────────────────
# 結果サマリ
# ──────────────────────────────────────────────────────────
echo ""
if [[ "${FAIL}" -eq 1 ]]; then
  echo "[result] FAIL — 重複 IMP-* ID が検出されました"
  echo "         修正候補: ${LEDGER_FILE} および各章 90_対応索引"
  exit 1
else
  echo "[result] PASS — 重複 ID なし"
  exit 0
fi
