#!/usr/bin/env bash
# B 軸: 手抜き検出ロジック
#
# 判定基準: docs/00_format/audit_criteria.md §B 軸
# 出力先: ${EVIDENCE_DIR}/slack.txt （集計サマリ）
#         ${EVIDENCE_DIR}/slack-locations.txt （ファイルパス + 行番号）
#         ${EVIDENCE_DIR}/slack-scope.txt （走査範囲: ファイル数 / 除外）
#
# 設計原則:
#   - 走査範囲を必ず明示（不在の証明）
#   - パターンごとに件数とロケーションを分離
#   - 0 件の項目も「0 件」と明記（沈黙させない）
#   - 自動生成コード（_grpc.pb.go / Generated/ / gen/ 等）は除外
#     （UnimplementedXxxServer 等の生成 stub は手抜きではなく forward compat の仕組み）

set -euo pipefail
REPO_ROOT="$1"
EVIDENCE_DIR="$2"

SLACK_OUT="${EVIDENCE_DIR}/slack.txt"
LOC_OUT="${EVIDENCE_DIR}/slack-locations.txt"
SCOPE_OUT="${EVIDENCE_DIR}/slack-scope.txt"

# 走査対象: コード本体（src / infra / deploy / tools / tests / examples）
# 除外: 依存物（vendor / node_modules / target / dist / build / .git / .next / obj / bin）
#       生成コード（generated / gen / *_pb.go / *.pb.cs / *.pb.rs / Generated/）
#       lock / 自動生成資産（pnpm-lock.yaml / package-lock.json / Cargo.lock 等）
#       audit lib 自身（本ファイルが検出パターン定義として禁止語彙を含むため、self-detection 防止）
INCLUDE_PATHS=(src infra deploy tools tests examples)
EXCLUDE_DIRS_RE='/(vendor|node_modules|target|dist|generated|gen|Generated|\.git|build|\.next|obj|bin|audit/lib)/'
EXCLUDE_FILES_RE='(_grpc\.pb\.go|\.pb\.go|\.pb\.cs|\.pb\.rs|pnpm-lock\.yaml|package-lock\.json|Cargo\.lock|yarn\.lock|go\.sum)$'

# 走査対象のファイルリストを作る
build_filelist() {
  local list="${EVIDENCE_DIR}/.slack-filelist.txt"
  : > "${list}"
  for p in "${INCLUDE_PATHS[@]}"; do
    [[ -d "${REPO_ROOT}/${p}" ]] || continue
    find "${REPO_ROOT}/${p}" -type f \
      \( -name '*.go' -o -name '*.rs' -o -name '*.cs' -o -name '*.ts' -o -name '*.tsx' \
         -o -name '*.js' -o -name '*.jsx' -o -name '*.py' -o -name '*.sh' -o -name '*.yaml' \
         -o -name '*.yml' -o -name '*.toml' -o -name '*.json' -o -name '*.md' -o -name '*.proto' \
         -o -name '*.hbs' -o -name 'Dockerfile*' \) \
      2>/dev/null \
      | grep -Ev "${EXCLUDE_DIRS_RE}" \
      | grep -Ev "${EXCLUDE_FILES_RE}" \
      >> "${list}" || true
  done
  echo "${list}"
}

FILELIST="$(build_filelist)"
TOTAL_FILES="$(wc -l < "${FILELIST}" | tr -d ' ')"

{
  echo "# slack 軸 走査範囲"
  echo "scanned_paths: ${INCLUDE_PATHS[*]}"
  echo "excluded_dirs_regex: ${EXCLUDE_DIRS_RE}"
  echo "excluded_files_regex: ${EXCLUDE_FILES_RE}"
  echo "total_files: ${TOTAL_FILES}"
  echo "filelist: ${FILELIST}"
} > "${SCOPE_OUT}"

# パターン定義: 配列で持つ（label と pattern と grep_flags を 3 並びで）
# 注意: docs/ は走査対象外（説明文として禁止語彙を含むため）
# pattern 内に '|' を含めない（IFS split の罠回避）。代替表現で複数候補を表現する場合は (...|...) を使い、外側を ' ' で囲む
declare -a PATTERN_LABELS=()
declare -a PATTERN_REGEX=()
declare -a PATTERN_FLAGS=()

add_pattern() {
  PATTERN_LABELS+=("$1")
  PATTERN_REGEX+=("$2")
  PATTERN_FLAGS+=("${3:-}")
}

add_pattern "go-unimplemented"        'codes\.Unimplemented'                         ''
add_pattern "rust-unimplemented"      'unimplemented!\s*\('                          ''
add_pattern "rust-todo-macro"         'todo!\s*\('                                   ''
add_pattern "dotnet-notimplemented"   'NotImplementedException'                      ''
add_pattern "ts-not-impl"             'throw\s+new\s+Error\s*\(\s*["'\''][^"'\'']*not\s+impl' '-i'
add_pattern "python-not-impl"         'raise\s+NotImplementedError'                  ''
add_pattern "comment-todo"            '\bTODO\b'                                     ''
add_pattern "comment-fixme"           '\bFIXME\b'                                    ''
add_pattern "comment-xxx"             '\bXXX\b(?![-A-Z])'                            '-P'
add_pattern "ja-toriaezu"             'とりあえず'                                    ''
add_pattern "ja-zantei"               '暫定'                                          ''
add_pattern "ja-karioki"              '仮置き'                                        ''
add_pattern "ja-atode"                '(あとで|後で(?!ろ))'                           ''
add_pattern "en-for-now"              '\bfor now\b(?!\s*[:=,])'                      '-iP'
add_pattern "en-temporary"            '\btemporary\b'                                '-i'
add_pattern "en-quick-fix"            '\bquick fix\b'                                '-i'
add_pattern "en-hack-comment"         '(//|#)[[:space:]]*hack\b'                     '-i'
add_pattern "en-workaround"           '\bworkaround\b'                               '-i'
add_pattern "empty-catch-js"          'catch\s*\([^)]*\)\s*\{\s*\}'                  ''
add_pattern "empty-except-py"         'except[^:]*:\s*pass\b'                        ''
add_pattern "go-silent-err"           '_\s*=\s*err\b'                                ''
add_pattern "rust-unwrap-or-empty"    '\.unwrap_or\s*\(\s*\)'                        ''

: > "${SLACK_OUT}"
: > "${LOC_OUT}"

{
  echo "# slack 軸 集計（生成: $(date -Iseconds)）"
  echo "# 走査範囲: ${TOTAL_FILES} ファイル（${INCLUDE_PATHS[*]}）"
  echo "# 除外: 依存物 + 生成コード（_grpc.pb.go / *.pb.cs / generated/ / gen/ 等）"
  echo
} >> "${SLACK_OUT}"

n_patterns="${#PATTERN_LABELS[@]}"
for ((i=0; i<n_patterns; i++)); do
  label="${PATTERN_LABELS[$i]}"
  pattern="${PATTERN_REGEX[$i]}"
  flags="${PATTERN_FLAGS[$i]}"

  if [[ -n "${flags}" ]]; then
    matches="$(xargs --no-run-if-empty -a "${FILELIST}" -d '\n' grep -nE "${flags}" -- "${pattern}" 2>/dev/null || true)"
  else
    matches="$(xargs --no-run-if-empty -a "${FILELIST}" -d '\n' grep -nE -- "${pattern}" 2>/dev/null || true)"
  fi
  count="$(printf '%s\n' "${matches}" | grep -c . || true)"
  count="${count:-0}"

  printf '%s: %s\n' "${label}" "${count}" >> "${SLACK_OUT}"

  if [[ "${count}" -gt 0 ]]; then
    {
      echo "## ${label} (${count} 件)"
      printf '%s\n' "${matches}"
      echo
    } >> "${LOC_OUT}"
  fi
done

# .gitkeep のみのディレクトリを検出
gitkeep_only_count=0
gitkeep_only_dirs=""
while IFS= read -r d; do
  files_in_dir="$(find "${d}" -mindepth 1 -maxdepth 1 -type f 2>/dev/null | wc -l | tr -d ' ')"
  if [[ "${files_in_dir}" == "1" ]] && [[ -f "${d}/.gitkeep" ]]; then
    gitkeep_only_count=$((gitkeep_only_count + 1))
    gitkeep_only_dirs+="${d}"$'\n'
  fi
done < <(find "${REPO_ROOT}" -type d \
  -not -path "*/node_modules/*" -not -path "*/target/*" -not -path "*/.git/*" \
  -not -path "*/vendor/*" -not -path "*/dist/*" -not -path "*/generated/*" \
  -not -path "*/gen/*" -not -path "*/Generated/*" \
  2>/dev/null)

echo "gitkeep-only-dirs: ${gitkeep_only_count}" >> "${SLACK_OUT}"
if [[ "${gitkeep_only_count}" -gt 0 ]]; then
  {
    echo "## gitkeep-only-dirs (${gitkeep_only_count} 件)"
    printf '%s' "${gitkeep_only_dirs}"
    echo
  } >> "${LOC_OUT}"
fi

# サマリ表示
echo "=== slack 軸 集計 ==="
cat "${SLACK_OUT}"
echo
echo "走査範囲: ${SCOPE_OUT}"
echo "ロケーション: ${LOC_OUT}"
