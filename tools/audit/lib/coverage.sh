#!/usr/bin/env bash
# A 軸: 要求網羅検証（FR / NFR / DS / IMP / ADR）
#
# 判定基準: docs/00_format/audit_criteria.md §A 軸
# 出力: ${EVIDENCE_DIR}/ids-${kind}.txt （docs 内 ID 一覧）
#       ${EVIDENCE_DIR}/coverage-${kind}.txt （ID ごとの 3 段確認証跡マトリクス）
#       ${EVIDENCE_DIR}/orphans-adr.txt （ADR の場合のみ: コードから引用されているが ADR 未起票）
#
# 設計原則 (audit-protocol skill 準拠):
#   - ID の網羅は grep で機械的に列挙
#   - 各 ID について 3 段確認の証跡件数を集める:
#       (a) docs 定義あり (docs/ 内参照件数)
#       (b) 実装サンプルあり (src/infra/deploy/tools/examples 参照件数)
#       (c) 動作証跡あり: test 参照件数 + SHIP_STATUS キーワード共起 + k8s 証跡
#   - Claude は判定者ではない。判定列は空欄、人間が AUDIT.md で記入する
#   - 「事実ベースの分類」（3 段揃い候補 / impl 不在 / test 不在）までは Claude が書く

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
    # 旧形式 (ADR-0001 等、4 桁通し番号、3 件のみ歴史的に存在) と
    # 新形式 (ADR-DATA-001 等、ドメイン別分類) の両対応。
    # `[A-Z][A-Z0-9]*` で新形式の category 部を「先頭が大文字」に縛り、
    # `ADR-0001-istio` の 0001 が新形式側にマッチして曖昧化するのを防ぐ。
    ID_REGEX='ADR-([0-9]{4}|[A-Z][A-Z0-9]*-[0-9]+)'
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

# 実装側 / test 側の走査パス
IMPL_PATHS=(src infra deploy tools examples)
IMPL_PATHS_FULL=()
for p in "${IMPL_PATHS[@]}"; do
  [[ -d "${REPO_ROOT}/${p}" ]] && IMPL_PATHS_FULL+=("${REPO_ROOT}/${p}")
done

TEST_PATHS=(tests)
TEST_PATHS_FULL=()
for p in "${TEST_PATHS[@]}"; do
  [[ -d "${REPO_ROOT}/${p}" ]] && TEST_PATHS_FULL+=("${REPO_ROOT}/${p}")
done

# tests/ 配下に加えて src 内 test ファイル（Go の *_test.go 等）も test 証跡として扱う
SRC_TEST_FILES_TMP="${EVIDENCE_DIR}/.${KIND}-src-tests.tmp"
if [[ ${#IMPL_PATHS_FULL[@]} -gt 0 ]]; then
  find "${IMPL_PATHS_FULL[@]}" -type f \
    \( -name '*_test.go' -o -name '*_test.rs' -o -name '*.test.ts' -o -name '*.test.tsx' \
       -o -name '*.spec.ts' -o -name '*.spec.tsx' -o -name 'test_*.py' \
       -o -name 'Tests*.cs' -o -name '*Tests.cs' -o -name '*.Tests.cs' \) \
    2>/dev/null > "${SRC_TEST_FILES_TMP}" || true
else
  : > "${SRC_TEST_FILES_TMP}"
fi

# SHIP_STATUS の "検証済" "実機" "実 K8s" 系キーワードを含む行を事前抽出
SHIP_VERIFIED_TMP="${EVIDENCE_DIR}/.${KIND}-ship-verified.tmp"
if [[ -f "${REPO_ROOT}/docs/SHIP_STATUS.md" ]]; then
  grep -nE '検証済|実機|実 K8s|実 cluster|round-trip|PASS|grpcurl' \
    "${REPO_ROOT}/docs/SHIP_STATUS.md" 2>/dev/null > "${SHIP_VERIFIED_TMP}" || true
else
  : > "${SHIP_VERIFIED_TMP}"
fi

# 出力ヘッダ（マトリクス）
{
  echo "# ${KIND} 軸 ID 網羅 + 3 段確認証跡 (生成: $(date -Iseconds))"
  echo "# 走査: ${DOCS_PATH}, パターン: ${ID_REGEX}"
  echo "# 列挙 ID 数: ${ID_COUNT}"
  echo "# 走査範囲: docs / impl(${IMPL_PATHS[*]}) / tests(${TEST_PATHS[*]} + src 内 test ファイル)"
  echo "# SHIP_STATUS 動作証跡: docs/SHIP_STATUS.md 内で検証済 / 実機 / round-trip 等のキーワード共起"
  echo "# 判定列は空欄。Claude は分類材料のみ提示、PASS / PARTIAL / FAIL の判定は人間が AUDIT.md で行う"
  echo
  echo "id | docs_refs | impl_refs | test_refs | ship_verified_kw | classification_material"
  echo "---|----------:|----------:|----------:|------------------|---|---"
} > "${COVERAGE_OUT}"

count_grep() {
  # grep 結果の件数を返す（マッチなしでも 0 を返す）
  # 注意: `set -o pipefail` 環境で `|| echo 0` を末尾に付けると、pipeline 失敗時に
  # "0\n0" のような複数行出力になり後段の算術評価が壊れる。`|| true` で吸収する。
  local pattern="$1"
  shift
  if [[ $# -eq 0 ]]; then
    echo 0
    return
  fi
  local n
  n="$(grep -rln "${pattern}" "$@" \
    --exclude-dir=node_modules --exclude-dir=target --exclude-dir=vendor \
    --exclude-dir=dist --exclude-dir=generated --exclude-dir=gen --exclude-dir=.git \
    2>/dev/null | wc -l | tr -d ' ' || true)"
  echo "${n:-0}"
}

count_test_refs() {
  # tests/ 配下 + src 内 test ファイルの両方を test 証跡として数える
  local id="$1"
  local n_tests=0 n_src_tests=0
  if [[ ${#TEST_PATHS_FULL[@]} -gt 0 ]]; then
    n_tests="$(count_grep "${id}" "${TEST_PATHS_FULL[@]}")"
  fi
  if [[ -s "${SRC_TEST_FILES_TMP}" ]]; then
    n_src_tests="$(xargs --no-run-if-empty -a "${SRC_TEST_FILES_TMP}" -d '\n' \
      grep -l "${id}" 2>/dev/null | wc -l | tr -d ' ' || true)"
    n_src_tests="${n_src_tests:-0}"
  fi
  echo $((n_tests + n_src_tests))
}

count_ship_verified() {
  # SHIP_STATUS の 検証済 / 実機 / round-trip 系キーワードと、id または id の接頭辞が同行に共起する数
  local id="$1"
  local prefix
  # ID 接頭辞（FR-T1-STATE / NFR-E-AC / DS-SW-COMP / IMP-DIR-INFRA / ADR-DATA 等）
  prefix="$(echo "${id}" | sed -E 's/-[0-9]+$//')"
  if [[ ! -s "${SHIP_VERIFIED_TMP}" ]]; then
    echo 0
    return
  fi
  local n_id n_prefix
  # grep -c は 0 件マッチで exit 1 + stdout "0"。`|| echo 0` だと "0\n0" になる罠を回避
  n_id="$(grep -c "${id}" "${SHIP_VERIFIED_TMP}" 2>/dev/null || true)"
  n_prefix="$(grep -c "${prefix}" "${SHIP_VERIFIED_TMP}" 2>/dev/null || true)"
  n_id="${n_id:-0}"
  n_prefix="${n_prefix:-0}"
  # 個別 ID の方を優先（より specific）、なければ接頭辞ベース
  if [[ "${n_id}" -gt 0 ]]; then
    echo "${n_id}"
  else
    echo "${n_prefix}"
  fi
}

classify() {
  # Claude が PASS/FAIL を書かない。事実ベースの分類のみ提示
  local docs_refs="$1" impl_refs="$2" test_refs="$3" ship_verified="$4"
  if [[ "${docs_refs}" -gt 0 && "${impl_refs}" -gt 0 ]]; then
    if [[ "${test_refs}" -gt 0 || "${ship_verified}" -gt 0 ]]; then
      echo "3-stage-candidate (docs+impl+evidence)"
    else
      echo "2-stage (docs+impl, no test/ship evidence)"
    fi
  elif [[ "${docs_refs}" -gt 0 && "${impl_refs}" == "0" ]]; then
    echo "docs-only (impl 不在)"
  elif [[ "${docs_refs}" == "0" ]]; then
    echo "id-not-in-docs (orphan 候補)"
  else
    echo "uncategorized"
  fi
}

# 各 ID をマトリクス化
while IFS= read -r id; do
  [[ -z "${id}" ]] && continue

  # docs 内参照件数
  docs_refs="$(count_grep "${id}" "${REPO_ROOT}/docs" 2>/dev/null)"
  # impl 内参照件数
  impl_refs=0
  if [[ ${#IMPL_PATHS_FULL[@]} -gt 0 ]]; then
    impl_refs="$(count_grep "${id}" "${IMPL_PATHS_FULL[@]}")"
  fi
  # test 参照件数
  test_refs="$(count_test_refs "${id}")"
  # SHIP_STATUS 検証済キーワード共起件数
  ship_verified="$(count_ship_verified "${id}")"
  # 分類
  cls="$(classify "${docs_refs}" "${impl_refs}" "${test_refs}" "${ship_verified}")"

  echo "${id} | ${docs_refs} | ${impl_refs} | ${test_refs} | ${ship_verified} | ${cls}" >> "${COVERAGE_OUT}"
done < "${IDS_OUT}"

# 一時ファイル掃除
rm -f "${SRC_TEST_FILES_TMP}" "${SHIP_VERIFIED_TMP}"

# ADR の場合のみ orphan 検出（コード参照あり ∩ ADR ファイル無し）
if [[ "${KIND}" == "adr" ]]; then
  ORPHANS_OUT="${EVIDENCE_DIR}/orphans-adr.txt"
  CODE_REFS_TMP="${EVIDENCE_DIR}/.adr-code-refs.tmp"
  if [[ ${#IMPL_PATHS_FULL[@]} -gt 0 ]]; then
    grep -rohE "${ID_REGEX}" "${IMPL_PATHS_FULL[@]}" \
      --exclude-dir=node_modules --exclude-dir=target --exclude-dir=vendor \
      --exclude-dir=dist --exclude-dir=generated --exclude-dir=gen --exclude-dir=.git \
      2>/dev/null | sort -u > "${CODE_REFS_TMP}" || true
  else
    : > "${CODE_REFS_TMP}"
  fi
  ADR_FILE_IDS_TMP="${EVIDENCE_DIR}/.adr-file-ids.tmp"
  ls "${REPO_ROOT}/docs/02_構想設計/adr/" 2>/dev/null \
    | grep -oE 'ADR-([0-9]{4}|[A-Z][A-Z0-9]*-[0-9]+)' | sort -u > "${ADR_FILE_IDS_TMP}" || true
  comm -23 "${CODE_REFS_TMP}" "${ADR_FILE_IDS_TMP}" > "${ORPHANS_OUT}" || true
  ORPHAN_COUNT="$(wc -l < "${ORPHANS_OUT}" | tr -d ' ')"
  {
    echo
    echo "# ADR orphan 検出"
    echo "# コード参照あり ∩ ADR ファイル無し = ${ORPHAN_COUNT} 件"
  } >> "${COVERAGE_OUT}"
  rm -f "${CODE_REFS_TMP}" "${ADR_FILE_IDS_TMP}"
fi

# 集計サマリ（分類別件数）
{
  echo
  echo "## 分類別集計"
  awk -F' \\| ' '$1 ~ /^[A-Z]/ && NF >= 6 { print $6 }' "${COVERAGE_OUT}" \
    | sort | uniq -c | awk '{ print "- " $0 }'
} >> "${COVERAGE_OUT}"

echo "=== ${KIND} 軸 集計 ==="
echo "ID 数: ${ID_COUNT}"
echo "coverage 出力: ${COVERAGE_OUT}"
[[ "${KIND}" == "adr" ]] && echo "orphan: ${ORPHANS_OUT}"
echo "分類別:"
awk -F' \\| ' '$1 ~ /^[A-Z]/ && NF >= 6 { print $6 }' "${COVERAGE_OUT}" 2>/dev/null \
  | sort | uniq -c | awk '{ print "  " $0 }'
