#!/usr/bin/env bash
# A 軸補助: 双方向トレース監査（最適化版）
#
# 判定基準: docs/00_format/audit_criteria.md §A 軸（補助）
# 出力: ${EVIDENCE_DIR}/trace-${kind}.txt
#
# 動機:
#   coverage.sh は ID 文字列の grep で実装サンプル件数を測るが、NFR / DS / IMP は
#   業界標準の設計慣行で「ID をコードに直接埋め込まない」ため、grep ベースで
#   92-98% が「impl 不在」と分類される。本 script はそれを「grep 限界」で逃げず、
#   (a) 直接, (b) 検証アーティファクト, (c) 同居 docs の co-cite 経由
#   を集計して trace_status を出す。
#
# パフォーマンス設計:
#   - 1416 ID × N 回 grep ではなく、3 種類の grep -rH walk を 1 回ずつ実行し
#     awk で ID ごとに集計する。1416 × |files| → 3 × |files| でほぼ定数に。

set -euo pipefail
REPO_ROOT="$1"
EVIDENCE_DIR="$2"
KIND="$3"  # nfr | ds | imp

case "${KIND}" in
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
  *)
    echo "trace.sh: unknown kind: ${KIND}" >&2
    exit 2
    ;;
esac

IDS_OUT="${EVIDENCE_DIR}/ids-${KIND}.txt"
TRACE_OUT="${EVIDENCE_DIR}/trace-${KIND}.txt"

# coverage.sh が ids-${KIND}.txt を作っていなければ自分で作る
if [[ ! -s "${IDS_OUT}" ]]; then
  if [[ -d "${REPO_ROOT}/${DOCS_PATH}" ]]; then
    grep -rohE "${ID_REGEX}" "${REPO_ROOT}/${DOCS_PATH}" 2>/dev/null | sort -u > "${IDS_OUT}" || true
  else
    : > "${IDS_OUT}"
  fi
fi

ID_COUNT="$(wc -l < "${IDS_OUT}" | tr -d ' ')"

# 走査対象 path
IMPL_PATHS=(src infra deploy tools examples)
VERIFICATION_PATHS=(
  "infra/security/kyverno"
  "infra/security/policies"
  "infra/observability"
  "deploy/rollouts"
  "deploy/policies"
  "ops/sli-slo"
  "ops/slo"
  "ops/runbooks"
  "tests/contract"
  "tests/e2e"
  "tests/fuzz"
  "tests/integration"
)

IMPL_PATHS_FULL=()
for p in "${IMPL_PATHS[@]}"; do
  [[ -d "${REPO_ROOT}/${p}" ]] && IMPL_PATHS_FULL+=("${REPO_ROOT}/${p}")
done

VERIFICATION_PATHS_FULL=()
for p in "${VERIFICATION_PATHS[@]}"; do
  [[ -d "${REPO_ROOT}/${p}" ]] && VERIFICATION_PATHS_FULL+=("${REPO_ROOT}/${p}")
done

# coverage 結果から「impl_refs > 0 の FR / ADR 集合」を抽出
FR_IMPL_SET="${EVIDENCE_DIR}/.trace-${KIND}-fr-impl-set.tmp"
ADR_IMPL_SET="${EVIDENCE_DIR}/.trace-${KIND}-adr-impl-set.tmp"
: > "${FR_IMPL_SET}"
: > "${ADR_IMPL_SET}"

if [[ -f "${EVIDENCE_DIR}/coverage-fr.txt" ]]; then
  awk -F' \\| ' '$1 ~ /^FR-T1-/ && $3+0 > 0 { print $1 }' "${EVIDENCE_DIR}/coverage-fr.txt" \
    | sort -u > "${FR_IMPL_SET}" || true
fi
if [[ -f "${EVIDENCE_DIR}/coverage-adr.txt" ]]; then
  awk -F' \\| ' '$1 ~ /^ADR-/ && $3+0 > 0 { print $1 }' "${EVIDENCE_DIR}/coverage-adr.txt" \
    | sort -u > "${ADR_IMPL_SET}" || true
fi

# === 一括 grep（最適化の核心） ===

EXCL='--exclude-dir=node_modules --exclude-dir=target --exclude-dir=vendor --exclude-dir=dist --exclude-dir=generated --exclude-dir=gen --exclude-dir=.git'

# 1) direct: impl_paths で ID -> file 一覧（重複あり）→ ID ごとに unique file 数
DIRECT_RAW="${EVIDENCE_DIR}/.trace-${KIND}-direct-raw.tmp"
DIRECT_COUNT="${EVIDENCE_DIR}/.trace-${KIND}-direct-count.tmp"
: > "${DIRECT_RAW}"
if [[ ${#IMPL_PATHS_FULL[@]} -gt 0 ]]; then
  # shellcheck disable=SC2086
  grep -rHoE ${EXCL} "${ID_REGEX}" "${IMPL_PATHS_FULL[@]}" 2>/dev/null > "${DIRECT_RAW}" || true
fi
# file:ID をユニーク化、ID ごとに件数集計
awk -F: '{ key=$1 ":" $2; if (!seen[key]++) cnt[$2]++ } END { for (id in cnt) print id, cnt[id] }' \
  "${DIRECT_RAW}" | sort > "${DIRECT_COUNT}" 2>/dev/null || true

# 2) verify: 検証 path で同様
VERIFY_RAW="${EVIDENCE_DIR}/.trace-${KIND}-verify-raw.tmp"
VERIFY_COUNT="${EVIDENCE_DIR}/.trace-${KIND}-verify-count.tmp"
: > "${VERIFY_RAW}"
if [[ ${#VERIFICATION_PATHS_FULL[@]} -gt 0 ]]; then
  # shellcheck disable=SC2086
  grep -rHoE ${EXCL} "${ID_REGEX}" "${VERIFICATION_PATHS_FULL[@]}" 2>/dev/null > "${VERIFY_RAW}" || true
fi
awk -F: '{ key=$1 ":" $2; if (!seen[key]++) cnt[$2]++ } END { for (id in cnt) print id, cnt[id] }' \
  "${VERIFY_RAW}" | sort > "${VERIFY_COUNT}" 2>/dev/null || true

# 3) docs インデックス: file → 含まれる KIND ID + 同居 FR / ADR ID
#    DOCS_RAW: "file:ID" 形式（複数 ID パターンを 1 回の grep で）
DOCS_RAW="${EVIDENCE_DIR}/.trace-${KIND}-docs-raw.tmp"
COMBINED_REGEX="${ID_REGEX}|FR-T1-[A-Z]+-[0-9]+|ADR-([0-9]{4}|[A-Z][A-Z0-9]*-[0-9]+)"
grep -rHoE "${COMBINED_REGEX}" "${REPO_ROOT}/docs" 2>/dev/null > "${DOCS_RAW}" || true

# DOCS_RAW から ID -> file 一覧 / file -> FR list / file -> ADR list を作る
# 構造を 3 つの中間ファイルに分解
ID_TO_FILES="${EVIDENCE_DIR}/.trace-${KIND}-id-to-files.tmp"
FILE_TO_FRS="${EVIDENCE_DIR}/.trace-${KIND}-file-to-frs.tmp"
FILE_TO_ADRS="${EVIDENCE_DIR}/.trace-${KIND}-file-to-adrs.tmp"

# ID -> files (KIND の ID のみ)
case "${KIND}" in
  nfr) MY_RE='^.*:NFR-' ;;
  ds)  MY_RE='^.*:DS-' ;;
  imp) MY_RE='^.*:IMP-' ;;
esac

# 行: "file:ID" - ID が KIND ID にマッチするものだけ
grep -E "${MY_RE}" "${DOCS_RAW}" 2>/dev/null \
  | awk -F: '{ id=$2; file=$1; key=id "@" file; if (!seen[key]++) print id, file }' \
  | sort -u > "${ID_TO_FILES}" || true

# file -> FR (FR-T1-* のみ)
grep -E '^.*:FR-T1-' "${DOCS_RAW}" 2>/dev/null \
  | awk -F: '{ key=$1 ":" $2; if (!seen[key]++) print $1, $2 }' \
  | sort -u > "${FILE_TO_FRS}" || true

# file -> ADR (ADR-* のみ)
grep -E '^.*:ADR-' "${DOCS_RAW}" 2>/dev/null \
  | awk -F: '{ key=$1 ":" $2; if (!seen[key]++) print $1, $2 }' \
  | sort -u > "${FILE_TO_ADRS}" || true

# === 出力ヘッダ ===
{
  echo "# ${KIND} trace 監査 (生成: $(date -Iseconds))"
  echo "# ID 数: ${ID_COUNT}"
  echo "# 動機: grep ベース coverage が「ID 直引用しない」性質で低スコアになる問題への補正"
  echo "# 列定義:"
  echo "#   direct        = src/infra/deploy/tools/examples で ID 直接 grep ヒット (coverage と重複)"
  echo "#   verify        = infra/security/, infra/observability/, deploy/rollouts/, ops/, tests/{contract,e2e,fuzz} 内の ID 引用"
  echo "#   cocited_fr    = ID 言及 docs に同居する FR-T1-* のうち、impl_refs>0 の件数"
  echo "#   cocited_adr   = ID 言及 docs に同居する ADR-* のうち、impl_refs>0 の件数"
  echo "#   trace_status  = どの経路で reach できるかの集約"
  echo
  echo "id | direct | verify | cocited_fr | cocited_adr | trace_status"
  echo "---|-------:|-------:|-----------:|------------:|---"
} > "${TRACE_OUT}"

# === 各 ID のメトリクス算出 ===
# AWK で全 ID を一度に処理する（速い）
awk -v ids_file="${IDS_OUT}" \
    -v direct_count="${DIRECT_COUNT}" \
    -v verify_count="${VERIFY_COUNT}" \
    -v id_to_files="${ID_TO_FILES}" \
    -v file_to_frs="${FILE_TO_FRS}" \
    -v file_to_adrs="${FILE_TO_ADRS}" \
    -v fr_impl_set="${FR_IMPL_SET}" \
    -v adr_impl_set="${ADR_IMPL_SET}" \
    'BEGIN {
       # FR / ADR impl 集合をハッシュにロード
       while ((getline line < fr_impl_set) > 0) fr_impl[line] = 1
       close(fr_impl_set)
       while ((getline line < adr_impl_set) > 0) adr_impl[line] = 1
       close(adr_impl_set)
       # direct / verify count をロード（"ID count" 形式）
       while ((getline line < direct_count) > 0) {
         split(line, a, " ")
         direct[a[1]] = a[2]
       }
       close(direct_count)
       while ((getline line < verify_count) > 0) {
         split(line, a, " ")
         verify[a[1]] = a[2]
       }
       close(verify_count)
       # id_to_files: "ID file" → id_files[ID] = "f1\nf2\n..."
       while ((getline line < id_to_files) > 0) {
         split(line, a, " ")
         id_files[a[1]] = id_files[a[1]] " " a[2]
       }
       close(id_to_files)
       # file_to_frs / file_to_adrs: "file ID" → file_frs[file] = " ID1 ID2 ..."
       while ((getline line < file_to_frs) > 0) {
         split(line, a, " ")
         file_frs[a[1]] = file_frs[a[1]] " " a[2]
       }
       close(file_to_frs)
       while ((getline line < file_to_adrs) > 0) {
         split(line, a, " ")
         file_adrs[a[1]] = file_adrs[a[1]] " " a[2]
       }
       close(file_to_adrs)
     }
     {
       id = $0
       d = (id in direct) ? direct[id] : 0
       v = (id in verify) ? verify[id] : 0
       cocited_fr_set = ""
       cocited_adr_set = ""
       # ID が言及される全 file 集合を取り、co-cite を集計
       split(id_files[id], files, " ")
       delete fr_seen
       delete adr_seen
       for (i in files) {
         f = files[i]
         if (f == "") continue
         # この file の FR たち
         split(file_frs[f], frs, " ")
         for (j in frs) if (frs[j] != "" && (frs[j] in fr_impl) && !fr_seen[frs[j]]++) cnt_fr++
         # この file の ADR たち
         split(file_adrs[f], adrs, " ")
         for (j in adrs) if (adrs[j] != "" && (adrs[j] in adr_impl) && !adr_seen[adrs[j]]++) cnt_adr++
       }
       cocited_fr_n = 0; for (k in fr_seen) cocited_fr_n++
       cocited_adr_n = 0; for (k in adr_seen) cocited_adr_n++
       # status
       labels = ""
       if (d > 0)            labels = labels "+direct"
       if (v > 0)            labels = labels "+verify"
       if (cocited_fr_n > 0) labels = labels "+via-fr"
       if (cocited_adr_n > 0) labels = labels "+via-adr"
       if (labels == "") status = "unreached"
       else { sub(/^\+/, "", labels); status = "reach(" labels ")" }
       printf "%s | %d | %d | %d | %d | %s\n", id, d, v, cocited_fr_n, cocited_adr_n, status
       cnt_fr = 0; cnt_adr = 0
     }' "${IDS_OUT}" >> "${TRACE_OUT}"

# 集計
{
  echo
  echo "## ${KIND} trace_status 別集計"
  awk -F' \\| ' '$1 ~ /^[A-Z]/ && NF >= 6 { print $6 }' "${TRACE_OUT}" \
    | sort | uniq -c | awk '{ print "- " $0 }'
  echo
  echo "## 解釈ガイド"
  echo "- direct>0: coverage.sh と同じ「ID 直引用」（NFR/DS/IMP では稀）"
  echo "- verify>0: policy / SLO / rollout / contract test に ID 引用 (推奨される検証経路)"
  echo "- via-fr / via-adr: ID 言及 docs と同じ docs に impl 済 FR/ADR が同居 (間接 reach)"
  echo "- unreached: docs 上では起票されているが、上記 4 経路いずれにも reach できない (要 inspect)"
} >> "${TRACE_OUT}"

# 一時ファイル掃除
rm -f "${FR_IMPL_SET}" "${ADR_IMPL_SET}" "${DIRECT_RAW}" "${DIRECT_COUNT}" \
      "${VERIFY_RAW}" "${VERIFY_COUNT}" "${DOCS_RAW}" \
      "${ID_TO_FILES}" "${FILE_TO_FRS}" "${FILE_TO_ADRS}"

echo "=== ${KIND} trace 軸 集計 ==="
echo "ID 数: ${ID_COUNT}"
echo "trace 出力: ${TRACE_OUT}"
echo "trace_status 別:"
awk -F' \\| ' '$1 ~ /^[A-Z]/ && NF >= 6 { print $6 }' "${TRACE_OUT}" 2>/dev/null \
  | sort | uniq -c | awk '{ print "  " $0 }'
