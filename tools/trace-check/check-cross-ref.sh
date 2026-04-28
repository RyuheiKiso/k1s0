#!/usr/bin/env bash
#
# tools/trace-check/check-cross-ref.sh — 90_対応索引と台帳の相互整合検証
#
# 設計: docs/05_実装/99_索引/50_整合性CI/01_整合性CI設計.md
# 関連 IMP-ID: IMP-TRACE-CI-012 / IMP-TRACE-CI-010
# 責務:
#   各章 90_対応IMP-XXX索引/ の合計値と 99_索引/00_IMP-ID一覧/ 台帳サマリ行を突き合わせ、
#   片方にしかない ID（90_索引のみ / 台帳のみ）を FAIL として報告する。
#   - 90_対応索引 → 台帳：90_索引に登場する ID が台帳に存在するか
#   - 台帳 → 90_対応索引：台帳の ID が少なくとも 1 つの 90_索引に存在するか（warning）
#
# Usage:
#   tools/trace-check/check-cross-ref.sh [--strict] [--help]
#
# Exit code:
#   0 = pass
#   1 = fail（片方にしかない ID あり）
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
      sed -n '3,20p' "$0" | sed 's/^# \{0,1\}//'
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
for cmd in grep find sort comm; do
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
# 台帳から全 ID を抽出（詳細行 + 範囲記法を展開）
#
# 台帳は採番効率のため `IMP-XXX-YYY-NNN〜MMM` 形式の範囲記法を多用する。
# 単純な regex 抽出だと範囲の開始 ID しか拾えないため、Python で範囲展開する。
# ──────────────────────────────────────────────────────────
LEDGER_IDS_FILE="$(mktemp)"
trap 'rm -f "${LEDGER_IDS_FILE}"' EXIT

python3 - "${LEDGER_FILE}" >"${LEDGER_IDS_FILE}" <<'PYEOF'
import re
import sys

ledger_path = sys.argv[1]
range_re = re.compile(r"(IMP-[A-Z]+-[A-Z]+)-(\d{3})\s*[〜~-]\s*(\d{3})")
single_re = re.compile(r"\b(IMP-[A-Z]+-[A-Z]+-\d{3})\b")
ids = set()

with open(ledger_path, encoding="utf-8") as f:
    for line in f:
        # 範囲記法を先に展開（先に消費しないと single_re が範囲先頭だけ拾う）
        for m in range_re.finditer(line):
            prefix, start, end = m.group(1), int(m.group(2)), int(m.group(3))
            for i in range(start, end + 1):
                ids.add(f"{prefix}-{i:03d}")
            # 範囲の表記自体は line 内から削除（残り検出のため）
            line = line.replace(m.group(0), "")
        for m in single_re.finditer(line):
            ids.add(m.group(1))

for i in sorted(ids):
    print(i)
PYEOF

ledger_count="$(wc -l < "${LEDGER_IDS_FILE}" | tr -d ' ')"
echo "[info] 台帳 ID 総数（範囲展開後）: ${ledger_count}"

# ──────────────────────────────────────────────────────────
# 90_対応索引ファイルを全探索して ID を抽出
# ──────────────────────────────────────────────────────────
INDEX90_IDS_FILE="$(mktemp)"
trap 'rm -f "${LEDGER_IDS_FILE}" "${INDEX90_IDS_FILE}"' EXIT

# 各章の 90_対応IMP-XXX索引/ ディレクトリ配下の md ファイルを探す
find "${DOCS_IMPL}" \
  -type f \
  -name "*.md" \
  -path "*/90_対応*索引/*" \
  | while read -r idx_file; do
      grep -oE 'IMP-[A-Z]+-[A-Z]+-[0-9]{3}' "${idx_file}" || true
    done \
  | sort -u > "${INDEX90_IDS_FILE}"

idx90_count="$(wc -l < "${INDEX90_IDS_FILE}" | tr -d ' ')"
echo "[info] 90_対応索引 ID 総数: ${idx90_count}"

# ──────────────────────────────────────────────────────────
# comm による差分検出
# ──────────────────────────────────────────────────────────
# comm -23: INDEX90 にあって台帳にない（台帳未登録）
# comm -13: 台帳にあって INDEX90 にない（索引漏れ = warning）

echo ""
echo "=== 90_対応索引にあって台帳にない ID（台帳未登録 → FAIL） ==="
only_in_idx90="$(comm -23 "${INDEX90_IDS_FILE}" "${LEDGER_IDS_FILE}")"
if [[ -n "${only_in_idx90}" ]]; then
  echo "${only_in_idx90}" | while read -r id; do
    echo "  [FAIL] ${id}: 90_対応索引に記載があるが台帳に未登録"
  done
  FAIL=1
else
  echo "  なし（正常）"
fi

echo ""
echo "=== 台帳にあって全 90_対応索引から漏れている ID（警告） ==="
only_in_ledger="$(comm -13 "${INDEX90_IDS_FILE}" "${LEDGER_IDS_FILE}")"
warn_count=0
if [[ -n "${only_in_ledger}" ]]; then
  echo "${only_in_ledger}" | while read -r id; do
    echo "  [warn] ${id}: 台帳に登録済だが 90_対応索引から参照されていない"
  done
  warn_count="$(echo "${only_in_ledger}" | wc -l | tr -d ' ')"
  # --strict モードでは warning も FAIL に格上げ
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
  echo "[result] FAIL — 索引と台帳の間に ID 不一致があります"
  echo "         修正候補: ${LEDGER_FILE} および各章 90_対応索引"
  exit 1
else
  echo "[result] PASS — 90_対応索引と台帳の相互整合確認完了"
  if [[ -n "${only_in_ledger}" ]]; then
    echo "         warn: 台帳にあって索引から漏れている ID が ${warn_count} 件あります（--strict で FAIL）"
  fi
  exit 0
fi
