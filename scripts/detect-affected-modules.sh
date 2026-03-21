#!/usr/bin/env bash
# エラー発生時に即座に終了し、未定義変数をエラーとして扱い、パイプラインの途中エラーも検知する（M-26対応）
set -euo pipefail
# 変更されたファイルから影響を受けるモジュールを検出するスクリプト
# CI パイプラインでモジュール単位のビルド・テスト最適化に使用する
#
# modules.yaml を唯一の情報源として使用し、skip-ci: true や archived モジュールを除外する。
#
# 使用方法:
#   scripts/detect-affected-modules.sh [base-branch] [language]
#   base-branch: 比較対象のブランチ（デフォルト: main）
#   language: フィルタする言語（go, rust, ts, dart）。省略時は全言語

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
MODULES_YAML="${REPO_ROOT}/modules.yaml"

# 引数からベースブランチと言語フィルタを取得
BASE_BRANCH="${1:-main}"
LANG_FILTER="${2:-}"

# modules.yaml から skip-ci: true または status: archived のパスを取得する
# Python の yaml モジュールを使ってパースし、除外パスリストを生成する
SKIP_PATHS=()
if command -v python3 >/dev/null 2>&1 && [ -f "$MODULES_YAML" ]; then
  # PyYAML が利用可能かを事前チェックする。未インストールの場合は警告を出す。
  if ! python3 -c "import yaml" 2>/dev/null; then
    echo "::warning::PyYAML が未インストールです。skip-ci フィルタをスキップします。" >&2
  else
    mapfile -t SKIP_PATHS < <(python3 - "$MODULES_YAML" <<'PYEOF'
import sys, yaml

modules_yaml = sys.argv[1]
with open(modules_yaml) as f:
    data = yaml.safe_load(f)
for mod in data.get('modules', []):
    if mod.get('skip-ci', False) or mod.get('status') == 'archived':
        print(mod['path'])
PYEOF
    )
  fi
fi

# ベースブランチを明示的に fetch してマージベースを確実に取得する。
# CI 環境では shallow clone のためベースブランチが存在しない場合がある。
# C-03 対応: --depth=1 では浅すぎてマージベースが取得できないため --deepen=50 を使用する。
# squash merge や長い PR チェーンでも正確なマージベースを取得できる。
if ! git rev-parse --verify "origin/${BASE_BRANCH}" >/dev/null 2>&1; then
  git fetch origin "${BASE_BRANCH}" --deepen=50 2>/dev/null || true
fi

# マージベースを取得して正確な差分を算出する
# マージベース取得失敗時は HEAD~1 にフォールバックし、その旨を警告する
merge_base=$(git merge-base "origin/${BASE_BRANCH}" HEAD 2>/dev/null) || {
  echo "::warning::merge-base の取得に失敗しました。HEAD~1 にフォールバックします。" >&2
  merge_base=$(git rev-parse HEAD~1 2>/dev/null) || {
    echo "::error::差分の比較基準を取得できません。リポジトリの fetch-depth を確認してください。" >&2
    exit 1
  }
}

# 変更されたファイルの一覧を取得（マージベースとの差分）
changed_files=$(git diff --name-only "${merge_base}" HEAD)

# モジュールルートを特定するマニフェストファイル（言語ごと）
declare -A MANIFEST_FILES=(
  [go]="go.mod"
  [rust]="Cargo.toml"
  [ts]="package.json"
  [dart]="pubspec.yaml"
)

# 変更ファイルから最寄りのモジュールルートを探索する
# 親ディレクトリを遡ってマニフェストファイルを探すが、
# ワークスペースルートやprotoディレクトリなど誤検出しやすいパスはスキップする
find_module_root() {
  local file="$1"
  local dir
  dir=$(dirname "$file")

  # ディレクトリを親方向へ遡りながらマニフェストファイルを探す
  while [ "$dir" != "." ] && [ "$dir" != "/" ]; do
    for lang in go rust ts dart; do
      # 言語フィルタが指定されている場合、対象外の言語はスキップ
      if [ -n "$LANG_FILTER" ] && [ "$lang" != "$LANG_FILTER" ]; then
        continue
      fi
      manifest="${MANIFEST_FILES[$lang]}"
      if [ -f "${dir}/${manifest}" ]; then
        # ワークスペースルート等はスキップ（誤検出防止）
        # - regions/system, CLI, CLI/crates/k1s0-gui: ワークスペースルートのマニフェスト
        # - api/proto, api: proto定義ディレクトリであり言語モジュールではない
        # - CLI/crates/k1s0-gui/regions/*: GUIに埋め込まれたテンプレート/フィクスチャ
        # - infra/*: インフラ設定ディレクトリ
        case "${dir}" in
          regions/system|CLI|CLI/crates/k1s0-gui|api/proto|api|infra/*)
            ;;
          CLI/crates/k1s0-gui/regions/*)
            ;;
          *)
            # modules.yaml で skip-ci: true または archived と指定されたパスを除外する
            skip=false
            for skip_path in "${SKIP_PATHS[@]:-}"; do
              if [ "${dir}" = "${skip_path}" ] || [[ "${dir}" == "${skip_path}/"* ]]; then
                skip=true
                break
              fi
            done
            if [ "$skip" = "false" ]; then
              echo "${lang}:${dir}"
            fi
            return
            ;;
        esac
      fi
    done
    dir=$(dirname "$dir")
  done
}

# 重複を排除して影響モジュールを出力
declare -A seen
while IFS= read -r file; do
  # 空行はスキップ
  [ -z "$file" ] && continue
  result=$(find_module_root "$file")
  if [ -n "$result" ] && [ -z "${seen[$result]:-}" ]; then
    seen[$result]=1
    echo "$result"
  fi
done <<< "$changed_files"
