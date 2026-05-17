#!/usr/bin/env bash
# tools/docs_lint/run_lint.sh
# ドキュメント lint メインスクリプト
# 7 check を順次実行、1 つでも fail で exit 1

set -euo pipefail

DOCS_DIR="${DOCS_DIR:-docs}"
EXIT_CODE=0

echo "=== docs_lint: 7 check 実行 ==="

# ---- 1. frontmatter schema 検査 ----
echo ""
echo "[1/7] frontmatter schema 検査"
# 全 .md の frontmatter を抽出し、required / forbidden / lock_artifacts pattern を検査
FORBIDDEN_FIELDS="^(changelog|last_updated|last_modified|author|reviewers):"
LOCK_PATTERN='^[a-z][a-z0-9_]*\.lock\.yaml$'
REQUIRED_FIELDS=("id" "axis" "phase" "kind" "status" "depends_on" "covered_by")

while IFS= read -r -d '' f; do
  # frontmatter 抽出（先頭 --- から次の --- まで）
  fm=$(awk '/^---$/{n++} n==1 && !/^---$/ {print} n==2{exit}' "$f")
  if [ -z "$fm" ]; then
    echo "  FAIL: $f frontmatter 不在"
    EXIT_CODE=1
    continue
  fi
  # forbidden field
  if echo "$fm" | grep -qE "$FORBIDDEN_FIELDS"; then
    echo "  FAIL: $f forbidden frontmatter field"
    EXIT_CODE=1
  fi
  # required field
  for field in "${REQUIRED_FIELDS[@]}"; do
    if ! echo "$fm" | grep -qE "^$field:"; then
      echo "  FAIL: $f required field '$field' 不在"
      EXIT_CODE=1
    fi
  done
  # lock_artifacts pattern
  if echo "$fm" | grep -qE "^lock_artifacts:"; then
    locks=$(echo "$fm" | awk '/^lock_artifacts:/{flag=1; next} /^[a-z]/{flag=0} flag && /^  - / {gsub(/^  - /, ""); print}')
    while IFS= read -r lock; do
      [ -z "$lock" ] && continue
      if ! echo "$lock" | grep -qE "$LOCK_PATTERN"; then
        echo "  FAIL: $f lock_artifacts '$lock' pattern 違反"
        EXIT_CODE=1
      fi
    done <<< "$locks"
  fi
done < <(find "$DOCS_DIR" -name '*.md' -not -path '*/90_archive/*' -not -path '*/00_format/*' -not -path "$DOCS_DIR/README.md" -print0)

# ---- 2. id 一意性検査 ----
echo ""
echo "[2/7] id 一意性検査"
ids_file=$(mktemp)
while IFS= read -r -d '' f; do
  id=$(awk '/^id:/{print $2; exit}' "$f")
  [ -n "$id" ] && echo "$id $f" >> "$ids_file"
done < <(find "$DOCS_DIR" -name '*.md' -not -path '*/90_archive/*' -not -path '*/00_format/*' -not -path "$DOCS_DIR/README.md" -print0)

dups=$(awk '{print $1}' "$ids_file" | sort | uniq -d)
if [ -n "$dups" ]; then
  echo "  FAIL: id 重複検出"
  echo "$dups" | while read -r d; do
    echo "    duplicate id: $d"
    grep "^$d " "$ids_file" | awk '{print "      " $2}'
  done
  EXIT_CODE=1
fi
rm -f "$ids_file"

# ---- 3. id 導出整合 ----
echo ""
echo "[3/7] id 導出整合（簡易チェック、phase prefix のみ）"
while IFS= read -r -d '' f; do
  id=$(awk '/^id:/{print $2; exit}' "$f")
  [ -z "$id" ] && continue
  prefix=$(echo "$id" | cut -d. -f1)
  expected_phase=""
  case "$f" in
    */01_企画/*) expected_phase="plan" ;;
    */02_要件定義/*) expected_phase="req" ;;
    */03_概要設計/*) expected_phase="arch" ;;
    */04_詳細設計/*) expected_phase="detail" ;;
    */05_環境構築/*) expected_phase="env" ;;
  esac
  if [ -n "$expected_phase" ] && [ "$prefix" != "$expected_phase" ]; then
    echo "  FAIL: $f id prefix '$prefix' != expected '$expected_phase'"
    EXIT_CODE=1
  fi
done < <(find "$DOCS_DIR" -name '*.md' -not -path '*/90_archive/*' -not -path '*/00_format/*' -not -path "$DOCS_DIR/README.md" -print0)

# ---- 4. depends_on 参照整合 ----
echo ""
echo "[4/7] depends_on 参照整合"
all_ids=$(find "$DOCS_DIR" -name '*.md' -not -path '*/90_archive/*' -exec awk '/^id:/{print $2}' {} \;)
while IFS= read -r -d '' f; do
  fm=$(awk '/^---$/{n++} n==1 && !/^---$/ {print} n==2{exit}' "$f")
  deps=$(echo "$fm" | awk '/^depends_on:/{flag=1; next} /^[a-z]/{flag=0} flag && /^  - / {gsub(/^  - /, ""); print}')
  while IFS= read -r dep; do
    [ -z "$dep" ] && continue
    if ! echo "$all_ids" | grep -qFx "$dep"; then
      echo "  FAIL: $f depends_on '$dep' 解決不能"
      EXIT_CODE=1
    fi
  done <<< "$deps"
done < <(find "$DOCS_DIR" -name '*.md' -not -path '*/90_archive/*' -not -path '*/00_format/*' -not -path "$DOCS_DIR/README.md" -print0)

# ---- 5. cross-link dangling 検査 ----
echo ""
echo "[5/7] cross-link dangling 検査"
while IFS= read -r -d '' f; do
  dir=$(dirname "$f")
  links=$(grep -oE '\]\([^)]+\.md[^)]*\)' "$f" 2>/dev/null | sed 's/^\](//; s/)$//' | grep -v '^http' || true)
  while IFS= read -r link; do
    [ -z "$link" ] && continue
    # remove anchor
    link_path=$(echo "$link" | cut -d'#' -f1)
    [ -z "$link_path" ] && continue
    target="$dir/$link_path"
    if [ ! -f "$target" ]; then
      echo "  FAIL: $f dangling link: $link"
      EXIT_CODE=1
    fi
  done <<< "$links"
done < <(find "$DOCS_DIR" -name '*.md' -not -path '*/90_archive/*' -not -path '*/00_format/*' -not -path "$DOCS_DIR/README.md" -print0)

# ---- 6. 禁止表現検査 ----
echo ""
echo "[6/7] 禁止表現検査"
FORBIDDEN_EXPR='Phase [0-9]+ で.*Phase [0-9]+ で追記'
while IFS= read -r -d '' f; do
  # frontmatter 範囲外で禁止表現を check
  body=$(awk '/^---$/{n++; next} n>=2' "$f")
  if echo "$body" | grep -qE "$FORBIDDEN_EXPR"; then
    echo "  FAIL: $f 段階的 release 表現「$FORBIDDEN_EXPR」検出"
    EXIT_CODE=1
  fi
done < <(find "$DOCS_DIR" -name '*.md' -not -path '*/90_archive/*' -not -path '*/00_format/*' -not -path "$DOCS_DIR/README.md" -print0)

# ---- 7. 空セクション / TBD 残存検査（status: locked のみ） ----
echo ""
echo "[7/7] 空セクション / TBD 残存検査（status: locked のみ）"
while IFS= read -r -d '' f; do
  status=$(awk '/^status:/{print $2; exit}' "$f")
  [ "$status" != "locked" ] && continue
  # ## で始まる section の直後が空 / TBD のみであれば fail
  if grep -qE '(TBD|未定|後述のみ)' "$f"; then
    echo "  FAIL: $f locked status で TBD / 未定 / 後述のみ検出"
    EXIT_CODE=1
  fi
done < <(find "$DOCS_DIR" -name '*.md' -not -path '*/90_archive/*' -not -path '*/00_format/*' -not -path "$DOCS_DIR/README.md" -print0)

# ---- 結果 ----
echo ""
if [ $EXIT_CODE -eq 0 ]; then
  echo "=== docs_lint: 7 check 全 green ==="
else
  echo "=== docs_lint: FAIL detected ==="
fi
exit $EXIT_CODE
