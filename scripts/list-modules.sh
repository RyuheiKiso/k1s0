#!/usr/bin/env bash
# エラー発生時に即座に終了し、未定義変数をエラーとして扱い、パイプラインの途中エラーも検知する（M-26対応）
set -euo pipefail
# modules.yaml からフィルタ条件に合致するモジュールパスを出力するスクリプト
# CI・justfile のスキップリストを廃止し、このスクリプトで一元管理する
#
# 使用方法:
#   scripts/list-modules.sh [--lang LANG] [--status STATUS] [--type TYPE] [--skip-ci] [--no-skip-ci]
#
# 例:
#   scripts/list-modules.sh --lang rust --status stable --no-skip-ci
#   scripts/list-modules.sh --lang go --type server
#   scripts/list-modules.sh --status experimental

set -euo pipefail

# デフォルト値: フィルタなし（全モジュール出力）
LANG_FILTER=""
STATUS_FILTER=""
TYPE_FILTER=""
SKIP_CI_FILTER=""  # "true" | "false" | ""（未指定）

# 引数パース
while [[ $# -gt 0 ]]; do
  case "$1" in
    --lang)
      LANG_FILTER="$2"
      shift 2
      ;;
    --status)
      STATUS_FILTER="$2"
      shift 2
      ;;
    --type)
      TYPE_FILTER="$2"
      shift 2
      ;;
    --skip-ci)
      SKIP_CI_FILTER="true"
      shift
      ;;
    --no-skip-ci)
      SKIP_CI_FILTER="false"
      shift
      ;;
    *)
      echo "Unknown option: $1" >&2
      echo "Usage: $0 [--lang LANG] [--status STATUS] [--type TYPE] [--skip-ci] [--no-skip-ci]" >&2
      exit 1
      ;;
  esac
done

# modules.yaml のパスを特定する（リポジトリルートからの相対パス）
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
MODULES_FILE="$REPO_ROOT/modules.yaml"

# modules.yaml が存在しない場合はエラー
if [ ! -f "$MODULES_FILE" ]; then
  echo "::error::modules.yaml が見つかりません: $MODULES_FILE" >&2
  exit 1
fi

# yq が利用可能か確認し、なければ軽量な bash パーサーにフォールバック
if command -v yq >/dev/null 2>&1; then
  # yq フィルタ式を構築する
  filter='.modules[]'

  # 言語フィルタ
  if [ -n "$LANG_FILTER" ]; then
    filter="${filter} | select(.lang == \"${LANG_FILTER}\")"
  fi

  # ステータスフィルタ
  if [ -n "$STATUS_FILTER" ]; then
    filter="${filter} | select(.status == \"${STATUS_FILTER}\")"
  fi

  # タイプフィルタ
  if [ -n "$TYPE_FILTER" ]; then
    filter="${filter} | select(.type == \"${TYPE_FILTER}\")"
  fi

  # skip-ci フィルタ
  if [ "$SKIP_CI_FILTER" = "true" ]; then
    filter="${filter} | select(.\"skip-ci\" == true)"
  elif [ "$SKIP_CI_FILTER" = "false" ]; then
    filter="${filter} | select(.\"skip-ci\" != true)"
  fi

  filter="${filter} | .path"

  yq eval "$filter" "$MODULES_FILE"
else
  # yq 非依存のフォールバック: bash + grep/awk で簡易パース
  # modules.yaml の各エントリを "- path:" 行で区切り、フィルタ条件に合致するものを出力
  current_path=""
  current_lang=""
  current_status=""
  current_type=""
  current_skip=""

  # エントリの出力判定関数
  emit_if_match() {
    if [ -z "$current_path" ]; then
      return
    fi
    # 各フィルタ条件をチェック
    if [ -n "$LANG_FILTER" ] && [ "$current_lang" != "$LANG_FILTER" ]; then
      return
    fi
    if [ -n "$STATUS_FILTER" ] && [ "$current_status" != "$STATUS_FILTER" ]; then
      return
    fi
    if [ -n "$TYPE_FILTER" ] && [ "$current_type" != "$TYPE_FILTER" ]; then
      return
    fi
    if [ "$SKIP_CI_FILTER" = "true" ] && [ "$current_skip" != "true" ]; then
      return
    fi
    if [ "$SKIP_CI_FILTER" = "false" ] && [ "$current_skip" = "true" ]; then
      return
    fi
    echo "$current_path"
  }

  while IFS= read -r line; do
    # 新しいエントリの開始を検出
    if [[ "$line" =~ ^[[:space:]]*-[[:space:]]+path:[[:space:]]*(.+) ]]; then
      # 前のエントリを出力判定
      emit_if_match
      current_path="${BASH_REMATCH[1]}"
      current_lang=""
      current_status=""
      current_type=""
      current_skip=""
    elif [[ "$line" =~ ^[[:space:]]+lang:[[:space:]]*(.+) ]]; then
      current_lang="${BASH_REMATCH[1]}"
    elif [[ "$line" =~ ^[[:space:]]+status:[[:space:]]*(.+) ]]; then
      current_status="${BASH_REMATCH[1]}"
    elif [[ "$line" =~ ^[[:space:]]+type:[[:space:]]*(.+) ]]; then
      current_type="${BASH_REMATCH[1]}"
    elif [[ "$line" =~ ^[[:space:]]+skip-ci:[[:space:]]*(.+) ]]; then
      current_skip="${BASH_REMATCH[1]}"
    fi
  done < "$MODULES_FILE"

  # 最後のエントリを出力判定
  emit_if_match
fi
