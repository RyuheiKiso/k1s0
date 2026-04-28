#!/usr/bin/env bash
#
# tools/trace-check/check-grand-total.sh — 台帳サマリ vs 詳細行集計の突き合わせ
#
# 設計: docs/05_実装/99_索引/50_整合性CI/01_整合性CI設計.md
# 関連 IMP-ID: IMP-TRACE-CI-011 / IMP-TRACE-CI-010
# 責務:
#   台帳（01_IMP-ID台帳_全12接頭辞.md）のサマリ表に記載された
#   「採番済合計」「実装 ID」「POL」列の各値と、台帳内の詳細 ID 行数を突き合わせ、
#   ずれがある場合は FAIL を返す。
#   台帳行形式は「個別 ID（IMP-XXX-POL-001）」と「範囲 ID（IMP-XXX-POL-001〜007）」
#   の両方に対応する。
#
# Usage:
#   tools/trace-check/check-grand-total.sh [--strict] [--help]
#
# Exit code:
#   0 = pass（全集計一致）
#   1 = fail（集計ずれあり）
#   2 = setup error（依存ツール欠如 / ファイル不在）

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
LEDGER_FILE="${REPO_ROOT}/docs/05_実装/99_索引/00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md"
STRICT=0

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
if ! command -v python3 >/dev/null 2>&1; then
  echo "[setup-error] python3 が見つかりません" >&2
  exit 2
fi

if [[ ! -f "${LEDGER_FILE}" ]]; then
  echo "[setup-error] 台帳ファイルが見つかりません: ${LEDGER_FILE}" >&2
  exit 2
fi

# ──────────────────────────────────────────────────────────
# Python による集計（個別行 + 範囲行の両方に対応）
# ──────────────────────────────────────────────────────────
STRICT_ARG=""
[[ "${STRICT}" -eq 1 ]] && STRICT_ARG="--strict"

python3 - "${LEDGER_FILE}" ${STRICT_ARG:+${STRICT_ARG}} <<'PYEOF'
import sys, re

ledger_path = sys.argv[1]
strict = "--strict" in sys.argv

# 台帳から全接頭辞の詳細行を集計する
# 対応形式:
#   個別行: | IMP-BUILD-POL-001 | ...
#   範囲行: | IMP-DEP-POL-001〜007 | ...  → 7 件としてカウント
#   範囲行: | IMP-DEP-REN-010〜019 | ...  → 10 件としてカウント

SINGLE_RE = re.compile(r'^\| (IMP-([A-Z]+)-([A-Z]+)-(\d{3})) \|')
RANGE_RE  = re.compile(r'^\| (IMP-([A-Z]+)-([A-Z]+)-(\d{3})[〜~](\d{3})) \|')

# サマリ表の行: | IMP-BUILD | 10 ビルド | 7 | 15 | 22 | 77 | ...
SUMMARY_RE = re.compile(r'^\| (IMP-[A-Z]+) \| .+? \| (\d+) \| (\d+) \| (\d+) \| (\d+) \|')

with open(ledger_path) as f:
    lines = f.readlines()

# 接頭辞ごとに集計
actual = {}   # prefix -> {pol: N, impl: N}
summary = {}  # prefix -> {pol: N, impl: N, total: N}

for line in lines:
    line = line.rstrip()

    # サマリ行
    m = SUMMARY_RE.match(line)
    if m:
        prefix = m.group(1)
        pol_exp   = int(m.group(2))
        impl_exp  = int(m.group(3))
        total_exp = int(m.group(4))
        summary[prefix] = {"pol": pol_exp, "impl": impl_exp, "total": total_exp}
        if prefix not in actual:
            actual[prefix] = {"pol": 0, "impl": 0}
        continue

    # 範囲行（〜 または ~）
    m = RANGE_RE.match(line)
    if m:
        prefix_main = "IMP-" + m.group(2)
        sub = m.group(3)
        lo = int(m.group(4))
        hi = int(m.group(5))
        count = hi - lo + 1
        if prefix_main not in actual:
            actual[prefix_main] = {"pol": 0, "impl": 0}
        if sub == "POL":
            actual[prefix_main]["pol"] += count
        else:
            actual[prefix_main]["impl"] += count
        continue

    # 個別行
    m = SINGLE_RE.match(line)
    if m:
        prefix_main = "IMP-" + m.group(2)
        sub = m.group(3)
        if prefix_main not in actual:
            actual[prefix_main] = {"pol": 0, "impl": 0}
        if sub == "POL":
            actual[prefix_main]["pol"] += 1
        else:
            actual[prefix_main]["impl"] += 1
        continue

FAIL = False
PREFIXES = ["IMP-BUILD","IMP-CODEGEN","IMP-CI","IMP-DEP","IMP-DEV",
            "IMP-OBS","IMP-REL","IMP-SUP","IMP-SEC","IMP-POL","IMP-DX","IMP-TRACE"]

for prefix in PREFIXES:
    if prefix not in summary:
        print(f"[warn] {prefix}: サマリ行が見つかりません")
        continue

    exp = summary[prefix]
    act = actual.get(prefix, {"pol": 0, "impl": 0})
    act_total = act["pol"] + act["impl"]

    ok = True
    issues = []
    if act["pol"] != exp["pol"]:
        issues.append(f"POL 期待={exp['pol']} 実数={act['pol']}")
        ok = False
    if act["impl"] != exp["impl"]:
        issues.append(f"実装ID 期待={exp['impl']} 実数={act['impl']}")
        ok = False
    if act_total != exp["total"]:
        issues.append(f"採番済合計 期待={exp['total']} 実数={act_total}")
        ok = False

    if ok:
        print(f"[ok]   {prefix}: POL={act['pol']} 実装={act['impl']} 合計={act_total}")
    else:
        FAIL = True
        for issue in issues:
            print(f"[FAIL] {prefix}: {issue}")

print()
if FAIL:
    print("[result] FAIL — 台帳サマリと詳細行の集計値にずれがあります")
    print(f"         修正候補: {ledger_path}")
    sys.exit(1)
else:
    print("[result] PASS — 全接頭辞の集計一致")
    sys.exit(0)
PYEOF
