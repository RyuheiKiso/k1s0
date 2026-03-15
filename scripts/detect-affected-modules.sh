#!/usr/bin/env bash
# 変更されたファイルから影響を受けるモジュールを検出するスクリプト
# CI パイプラインでモジュール単位のビルド・テスト最適化に使用する
#
# 使用方法:
#   scripts/detect-affected-modules.sh [base-branch] [language]
#   base-branch: 比較対象のブランチ（デフォルト: main）
#   language: フィルタする言語（go, rust, ts, dart）。省略時は全言語

set -euo pipefail

# 引数からベースブランチと言語フィルタを取得
BASE_BRANCH="${1:-main}"
LANG_FILTER="${2:-}"

# 変更されたファイルの一覧を取得（マージベースとの差分、失敗時は直前コミットとの差分）
changed_files=$(git diff --name-only "${BASE_BRANCH}"...HEAD 2>/dev/null || git diff --name-only HEAD~1)

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
            echo "${lang}:${dir}"
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
