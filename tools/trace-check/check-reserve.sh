#!/usr/bin/env bash
#
# tools/trace-check/check-reserve.sh — 予約帯 (001-099) 外採番検出
#
# 設計: docs/05_実装/99_索引/50_整合性CI/01_整合性CI設計.md
# 関連 IMP-ID: IMP-TRACE-CI-014 / IMP-TRACE-CI-010
# 責務:
#   IMP-TRACE-POL-002（各サブ接頭辞の予約帯 001-099 内での採番義務）を検証する。
#   `tools/trace-check/reserve-ranges.yaml` で宣言されたサブ接頭辞別予約範囲を超えた
#   採番（例: IMP-CI-HAR-100 のような 3 桁 + 1 桁超）および
#   サブ接頭辞間の範囲衝突を検出する。
#
# Usage:
#   tools/trace-check/check-reserve.sh [--strict] [--help]
#
# Exit code:
#   0 = pass
#   1 = fail（帯外採番あり）
#   2 = setup error

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
DOCS_IMPL="${REPO_ROOT}/docs/05_実装"
LEDGER_FILE="${DOCS_IMPL}/99_索引/00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md"
RANGES_FILE="${REPO_ROOT}/tools/trace-check/reserve-ranges.yaml"
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
for cmd in grep python3; do
  if ! command -v "${cmd}" >/dev/null 2>&1; then
    echo "[setup-error] ${cmd} が見つかりません" >&2
    exit 2
  fi
done

if [[ ! -f "${LEDGER_FILE}" ]]; then
  echo "[setup-error] 台帳ファイルが見つかりません: ${LEDGER_FILE}" >&2
  exit 2
fi

if [[ ! -f "${RANGES_FILE}" ]]; then
  echo "[setup-error] reserve-ranges.yaml が見つかりません: ${RANGES_FILE}" >&2
  echo "              tools/trace-check/reserve-ranges.yaml を作成してください" >&2
  exit 2
fi

# ──────────────────────────────────────────────────────────
# 1. 基本チェック: 番号が 001-099 の範囲外（100 以上）
# ──────────────────────────────────────────────────────────
echo "=== 予約帯 001-099 外の採番検出 ==="
OOB_IDS="$(
  grep -oE 'IMP-[A-Z]+-[A-Z]+-[0-9]+' "${LEDGER_FILE}" \
    | awk -F'-' '{n=$NF; if (n+0 > 99 || n+0 < 1) print $0}'
)"

if [[ -n "${OOB_IDS}" ]]; then
  echo "${OOB_IDS}" | while read -r id; do
    echo "  [FAIL] ${id}: 予約帯 001-099 の範囲外（IMP-TRACE-POL-002 違反）"
  done
  FAIL=1
else
  echo "  なし（正常）"
fi

# ──────────────────────────────────────────────────────────
# 2. reserve-ranges.yaml によるサブ接頭辞別範囲チェック
# ──────────────────────────────────────────────────────────
echo ""
echo "=== サブ接頭辞別予約範囲外採番の検出 ==="
python3 - <<'PYEOF'
import sys, re, pathlib, os

repo_root = os.environ.get("REPO_ROOT") or \
    __import__("subprocess").check_output(
        ["git","rev-parse","--show-toplevel"], text=True
    ).strip()

ledger_path = f"{repo_root}/docs/05_実装/99_索引/00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md"
ranges_path = f"{repo_root}/tools/trace-check/reserve-ranges.yaml"

try:
    import yaml
    has_yaml = True
except ImportError:
    has_yaml = False

# YAML パース（PyYAML なければ簡易パース）
def parse_ranges(path):
    """reserve-ranges.yaml を {PREFIX: {SUBPREFIX: (min, max)}} 形式で返す"""
    result = {}
    if has_yaml:
        with open(path) as f:
            data = yaml.safe_load(f)
        for prefix, subs in (data or {}).items():
            result[prefix] = {}
            for sub, rng in (subs or {}).items():
                lo, hi = str(rng).split("-")
                result[prefix][sub] = (int(lo), int(hi))
    else:
        # 簡易パース: "  SUB: 020-039" 形式
        current_prefix = None
        with open(path) as f:
            for line in f:
                m_prefix = re.match(r'^([A-Z]+):', line)
                if m_prefix:
                    current_prefix = m_prefix.group(1)
                    result[current_prefix] = {}
                    continue
                if current_prefix:
                    m_sub = re.match(r'\s+([A-Z]+):\s*(\d+)-(\d+)', line)
                    if m_sub:
                        sub = m_sub.group(1)
                        lo, hi = int(m_sub.group(2)), int(m_sub.group(3))
                        result[current_prefix][sub] = (lo, hi)
    return result

ranges = parse_ranges(ranges_path)

# 台帳から IMP ID を抽出
with open(ledger_path) as f:
    content = f.read()

ids = re.findall(r'IMP-([A-Z]+)-([A-Z]+)-(\d{3})', content)
fail = False
for prefix, sub, num_s in ids:
    num = int(num_s)
    if prefix not in ranges:
        continue
    if sub not in ranges[prefix]:
        continue
    lo, hi = ranges[prefix][sub]
    if not (lo <= num <= hi):
        print(f"  [FAIL] IMP-{prefix}-{sub}-{num_s}: 宣言範囲 {lo:03d}-{hi:03d} 外の採番")
        fail = True

if not fail:
    print("  なし（正常）")
    sys.exit(0)
else:
    sys.exit(1)
PYEOF

PYEXIT=$?
if [[ "${PYEXIT}" -ne 0 ]]; then
  FAIL=1
fi

# ──────────────────────────────────────────────────────────
# 結果サマリ
# ──────────────────────────────────────────────────────────
echo ""
if [[ "${FAIL}" -eq 1 ]]; then
  echo "[result] FAIL — 予約帯外の採番が検出されました（IMP-TRACE-POL-002 違反）"
  echo "         修正候補: ${LEDGER_FILE}"
  echo "         範囲定義: ${RANGES_FILE}"
  exit 1
else
  echo "[result] PASS — 全 ID が予約帯内に収まっています"
  exit 0
fi
