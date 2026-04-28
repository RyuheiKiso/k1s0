#!/usr/bin/env bash
#
# tools/catalog-check/scan-offpath.sh — Off-Path component 検出
#
# 設計: docs/05_実装/99_索引/60_catalog-info検証/01_catalog-info検証設計.md
# 関連 IMP-ID: IMP-TRACE-CAT-026 / IMP-TRACE-CAT-029 / IMP-DX-SCAF-033
# 責務:
#   src/ 配下のサービスディレクトリ（tier1/tier2/tier3）を走査し、
#   catalog-info.yaml が存在しない component（Off-Path）を検出する。
#   .k1s0-no-catalog ファイルが存在するディレクトリは除外する。
#   IMP-DX-SCAF-033（月次 Backstage Catalog 走査）と同一バイナリを共有する。
#
# Usage:
#   tools/catalog-check/scan-offpath.sh [--strict] [--report] [--help]
#
# Exit code:
#   0 = pass（Off-Path なし）
#   1 = fail（Off-Path あり。--strict 時のみ exit 1、通常は warn で exit 0）
#   2 = setup error

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
SRC_DIR="${REPO_ROOT}/src"
ALLOWLIST_FILE="${REPO_ROOT}/tools/catalog-check/offpath-allowlist.yaml"
STRICT=0
REPORT=0
FAIL=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --strict) STRICT=1; shift ;;
    --report) REPORT=1; shift ;;
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

if [[ ! -d "${SRC_DIR}" ]]; then
  echo "[setup-error] src/ ディレクトリが見つかりません: ${SRC_DIR}" >&2
  exit 2
fi

# ──────────────────────────────────────────────────────────
# Off-Path 許可リストを読み込む
# ──────────────────────────────────────────────────────────
ALLOWLIST_DIRS=()
if [[ -f "${ALLOWLIST_FILE}" ]]; then
  while IFS= read -r line; do
    stripped="${line%%#*}"  # コメント除去
    stripped="${stripped#"${stripped%%[![:space:]]*}"}"  # 先頭空白除去
    stripped="${stripped%"${stripped##*[![:space:]]}"}"  # 末尾空白除去
    [[ -n "${stripped}" ]] && ALLOWLIST_DIRS+=("${stripped}")
  done < <(grep -v '^#' "${ALLOWLIST_FILE}" | grep -v '^\s*$' || true)
fi

is_in_allowlist() {
  local dir="$1"
  local rel_path="${dir#"${REPO_ROOT}/"}"
  for allowed in "${ALLOWLIST_DIRS[@]:-}"; do
    if [[ "${rel_path}" == "${allowed}" ]] || [[ "${rel_path}" == "${allowed}/"* ]]; then
      return 0
    fi
  done
  return 1
}

# ──────────────────────────────────────────────────────────
# Off-Path 検出
# サービスディレクトリ = src/tier*/*/services/*/ または src/tier*/*/apps/*/ など
# catalog-info.yaml を持つべき「コンポーネントルート」を探す
# ──────────────────────────────────────────────────────────
OFFPATH_COUNT=0
OFFPATH_LIST=()
TOTAL_COUNT=0

# catalog-info.yaml を持つ既存コンポーネントと、
# 同レベルに存在するが持たないディレクトリを比較する
# strategy: catalog-info.yaml が 1 件以上存在する深さレベルの兄弟ディレクトリも対象とする

# src/ 配下でサービスらしいディレクトリ（2〜4 段のディレクトリを候補とする）
while IFS= read -r candidate_dir; do
  # .k1s0-no-catalog が存在する → 意図的 Off-Path のため除外
  if [[ -f "${candidate_dir}/.k1s0-no-catalog" ]]; then
    continue
  fi

  # allowlist にある → 除外
  if is_in_allowlist "${candidate_dir}"; then
    continue
  fi

  rel="${candidate_dir#"${REPO_ROOT}/"}"
  TOTAL_COUNT=$((TOTAL_COUNT + 1))

  if [[ ! -f "${candidate_dir}/catalog-info.yaml" ]]; then
    OFFPATH_COUNT=$((OFFPATH_COUNT + 1))
    OFFPATH_LIST+=("${rel}")
  fi
done < <(
  # catalog-info.yaml が 1 件以上存在するディレクトリの「親」を基準に
  # 同じ深さの兄弟ディレクトリを収集する
  find "${SRC_DIR}" -name "catalog-info.yaml" -type f \
    | xargs -I{} dirname {} \
    | xargs -I{} dirname {} \
    | sort -u \
    | while read -r parent; do
        find "${parent}" -mindepth 1 -maxdepth 1 -type d
      done \
    | sort -u
)

# ──────────────────────────────────────────────────────────
# 結果出力
# ──────────────────────────────────────────────────────────
echo "[info] スキャン対象ディレクトリ数: ${TOTAL_COUNT}"
echo "[info] Off-Path 検出数: ${OFFPATH_COUNT}"
echo ""

if [[ "${#OFFPATH_LIST[@]}" -gt 0 ]]; then
  echo "=== Off-Path component（catalog-info.yaml なし）==="
  for dir in "${OFFPATH_LIST[@]}"; do
    echo "  [warn] ${dir}"
  done
  echo ""
  echo "  除外方法: ディレクトリに .k1s0-no-catalog ファイルを置く"
  echo "  または:   ${ALLOWLIST_FILE} に追加する"

  if [[ "${STRICT}" -eq 1 ]]; then
    FAIL=1
  fi
fi

if [[ "${REPORT}" -eq 1 ]]; then
  REPORT_FILE="${REPO_ROOT}/offpath-report.md"
  {
    echo "# Off-Path Report"
    echo ""
    echo "生成日時: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo ""
    echo "| ディレクトリ | 状態 |"
    echo "|---|---|"
    for dir in "${OFFPATH_LIST[@]}"; do
      echo "| \`${dir}\` | Off-Path（catalog-info.yaml なし）|"
    done
  } > "${REPORT_FILE}"
  echo "[info] レポート出力: ${REPORT_FILE}"
fi

echo ""
if [[ "${FAIL}" -eq 1 ]]; then
  echo "[result] FAIL (strict) — Off-Path component が ${OFFPATH_COUNT} 件あります（IMP-TRACE-CAT-026）"
  exit 1
else
  echo "[result] PASS — Off-Path スキャン完了"
  if [[ "${OFFPATH_COUNT}" -gt 0 ]]; then
    echo "         warn: Off-Path ${OFFPATH_COUNT} 件（月次 cron で Sev3 通知対象: CAT-029）"
  fi
  exit 0
fi
