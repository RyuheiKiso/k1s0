#!/usr/bin/env bash
# A 軸: 要求網羅検証（FR / NFR / DS / IMP / ADR）
#
# 判定基準: docs/00_format/audit_criteria.md §A 軸
# 出力: ${EVIDENCE_DIR}/ids-${kind}.txt （docs 内 ID 一覧）
#       ${EVIDENCE_DIR}/coverage-${kind}.txt （ID ごとの実装サンプル参照件数）
#       ${EVIDENCE_DIR}/orphans-adr.txt （ADR の場合のみ: コードから引用されているが ADR 未起票）
#
# 設計原則:
#   - ID の網羅は grep で機械的に列挙
#   - 各 ID について src/infra/deploy/tests/examples で参照件数を出す
#   - 参照 0 件 = 実装欠落の可能性、判定は人間に委ねる

set -euo pipefail
REPO_ROOT="$1"
EVIDENCE_DIR="$2"
KIND="$3"

case "${KIND}" in
  fr)
    DOCS_PATH="docs/03_要件定義"
    ID_REGEX='FR-T1-[A-Z]+-[0-9]+'
    ;;
  nfr)
    DOCS_PATH="docs/03_要件定義"
    ID_REGEX='NFR-[A-I]-[A-Z]+-[0-9]+'
    ;;
  ds)
    DOCS_PATH="docs/04_概要設計"
    ID_REGEX='DS-[A-Z]+-[A-Z]+-[0-9]+'
    ;;
  imp)
    DOCS_PATH="docs/05_実装"
    ID_REGEX='IMP-[A-Z]+-[A-Z]+-[0-9]+'
    ;;
  adr)
    DOCS_PATH="docs/02_構想設計/adr"
    ID_REGEX='ADR-[A-Z0-9]+-[0-9]+'
    ;;
  *)
    echo "unknown kind: ${KIND}" >&2
    exit 2
    ;;
esac

IDS_OUT="${EVIDENCE_DIR}/ids-${KIND}.txt"
COVERAGE_OUT="${EVIDENCE_DIR}/coverage-${KIND}.txt"

# docs 内 ID 列挙
if [[ -d "${REPO_ROOT}/${DOCS_PATH}" ]]; then
  grep -rohE "${ID_REGEX}" "${REPO_ROOT}/${DOCS_PATH}" 2>/dev/null | sort -u > "${IDS_OUT}" || true
else
  : > "${IDS_OUT}"
fi

ID_COUNT="$(wc -l < "${IDS_OUT}" | tr -d ' ')"

{
  echo "# ${KIND} 軸 ID 網羅 (生成: $(date -Iseconds))"
  echo "# 走査: ${DOCS_PATH}, パターン: ${ID_REGEX}"
  echo "# 列挙 ID 数: ${ID_COUNT}"
  echo
  echo "id | impl_refs | locations_truncated"
  echo "---|-----------|--------------------"
} > "${COVERAGE_OUT}"

# 各 ID について実装側の参照件数を出す
SCAN_PATHS=(src infra deploy tests examples tools)
SCAN_PATHS_FULL=()
for p in "${SCAN_PATHS[@]}"; do
  [[ -d "${REPO_ROOT}/${p}" ]] && SCAN_PATHS_FULL+=("${REPO_ROOT}/${p}")
done

while IFS= read -r id; do
  [[ -z "${id}" ]] && continue
  if [[ ${#SCAN_PATHS_FULL[@]} -gt 0 ]]; then
    refs="$(grep -rln "${id}" "${SCAN_PATHS_FULL[@]}" \
      --exclude-dir=node_modules --exclude-dir=target --exclude-dir=vendor \
      --exclude-dir=dist --exclude-dir=generated --exclude-dir=.git \
      2>/dev/null | head -3 | tr '\n' ',' | sed 's/,$//' || true)"
    refs_count="$(grep -rln "${id}" "${SCAN_PATHS_FULL[@]}" \
      --exclude-dir=node_modules --exclude-dir=target --exclude-dir=vendor \
      --exclude-dir=dist --exclude-dir=generated --exclude-dir=.git \
      2>/dev/null | wc -l | tr -d ' ' || echo 0)"
  else
    refs=""
    refs_count=0
  fi
  echo "${id} | ${refs_count} | ${refs}" >> "${COVERAGE_OUT}"
done < "${IDS_OUT}"

# ADR の場合のみ orphan 検出
if [[ "${KIND}" == "adr" ]]; then
  ORPHANS_OUT="${EVIDENCE_DIR}/orphans-adr.txt"
  # コードから引用されている ADR ID
  CODE_REFS_TMP="${EVIDENCE_DIR}/.adr-code-refs.tmp"
  if [[ ${#SCAN_PATHS_FULL[@]} -gt 0 ]]; then
    grep -rohE "${ID_REGEX}" "${SCAN_PATHS_FULL[@]}" \
      --exclude-dir=node_modules --exclude-dir=target --exclude-dir=vendor \
      --exclude-dir=dist --exclude-dir=generated --exclude-dir=.git \
      2>/dev/null | sort -u > "${CODE_REFS_TMP}" || true
  else
    : > "${CODE_REFS_TMP}"
  fi
  # docs 内に ADR ファイルが存在する ID を列挙
  ADR_FILE_IDS_TMP="${EVIDENCE_DIR}/.adr-file-ids.tmp"
  ls "${REPO_ROOT}/docs/02_構想設計/adr/" 2>/dev/null \
    | grep -oE 'ADR-[A-Z0-9]+-[0-9]+' | sort -u > "${ADR_FILE_IDS_TMP}" || true

  # コード参照あり ∩ ADR ファイル無し = orphan
  comm -23 "${CODE_REFS_TMP}" "${ADR_FILE_IDS_TMP}" > "${ORPHANS_OUT}" || true
  ORPHAN_COUNT="$(wc -l < "${ORPHANS_OUT}" | tr -d ' ')"
  {
    echo
    echo "# ADR orphan 検出"
    echo "# コード参照あり ∩ ADR ファイル無し = ${ORPHAN_COUNT} 件"
  } >> "${COVERAGE_OUT}"

  rm -f "${CODE_REFS_TMP}" "${ADR_FILE_IDS_TMP}"
fi

echo "=== ${KIND} 軸 集計 ==="
echo "ID 数: ${ID_COUNT}"
echo "coverage 出力: ${COVERAGE_OUT}"
[[ "${KIND}" == "adr" ]] && echo "orphan: ${ORPHANS_OUT}"
