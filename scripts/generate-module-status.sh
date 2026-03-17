#!/usr/bin/env bash
# モジュールステータスを自動生成するスクリプト。
# 各サービスディレクトリのキーファイル（Cargo.toml, go.mod, Dockerfile 等）の
# 存在を確認し、成熟度レベルを判定して Markdown テーブルを出力する。
#
# 使用方法:
#   scripts/generate-module-status.sh [--output FILE]
#
# 例:
#   scripts/generate-module-status.sh
#   scripts/generate-module-status.sh --output docs/architecture/overview/module-status.md

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT=""

# 引数パース
while [[ $# -gt 0 ]]; do
  case "$1" in
    --output)
      OUTPUT="$2"
      shift 2
      ;;
    *)
      echo "Usage: $0 [--output FILE]" >&2
      exit 1
      ;;
  esac
done

# キーファイルの存在をチェックし、成熟度を判定する関数
# template-only: Cargo.toml/go.mod のみ
# experimental: ソースコードあり、テストなし
# beta: ソースコード + テスト + Dockerfile のいずれかが存在
# production: (手動昇格のため自動判定なし)
check_maturity() {
  local dir="$1"
  local has_tests=false
  local has_src=false
  local has_dockerfile=false
  local has_config=false

  # テストファイルの存在確認
  if [ -d "$dir/tests" ] || find "$dir/src" -name '*test*' -o -name '*_test.go' 2>/dev/null | head -1 | grep -q .; then
    has_tests=true
  fi

  # ソースコードの存在確認（scaffold 以上のファイル数）
  local src_count=0
  if [ -d "$dir/src" ]; then
    src_count=$(find "$dir/src" -type f \( -name '*.rs' -o -name '*.go' -o -name '*.ts' -o -name '*.dart' \) 2>/dev/null | wc -l)
  fi
  if [ "$src_count" -gt 3 ]; then
    has_src=true
  fi

  # Dockerfile の存在確認
  if [ -f "$dir/Dockerfile" ]; then
    has_dockerfile=true
  fi

  # config.yaml の存在確認
  if [ -f "$dir/config.yaml" ] || [ -f "$dir/config/config.yaml" ]; then
    has_config=true
  fi

  # 成熟度判定
  if $has_src && $has_tests; then
    echo "beta"
  elif $has_src; then
    echo "experimental"
  else
    echo "template-only"
  fi
}

# キーファイルの存在状況をサマリする関数
check_key_files() {
  local dir="$1"
  local files=""

  [ -f "$dir/Cargo.toml" ] && files="${files}Cargo.toml "
  [ -f "$dir/go.mod" ] && files="${files}go.mod "
  [ -f "$dir/package.json" ] && files="${files}package.json "
  [ -f "$dir/pubspec.yaml" ] && files="${files}pubspec.yaml "
  [ -f "$dir/Dockerfile" ] && files="${files}Dockerfile "
  [ -f "$dir/config.yaml" ] || [ -f "$dir/config/config.yaml" ] && files="${files}config.yaml "
  [ -d "$dir/tests" ] && files="${files}tests/ "

  echo "${files:-none}"
}

# 出力先の設定
if [ -n "$OUTPUT" ]; then
  exec > "$OUTPUT"
fi

# ヘッダー出力
cat <<'HEADER'
# モジュール成熟度ステータス一覧（自動生成）

<!-- このファイルは scripts/generate-module-status.sh により自動生成されます -->
HEADER

echo ""
echo "**生成日時**: $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
echo ""

# カウンター
count_production=0
count_beta=0
count_experimental=0
count_template=0

# --- System サーバー (Rust) ---
echo "## Systemティア — サーバー（Rust）"
echo ""
echo "| モジュール | 成熟度 | キーファイル |"
echo "|-----------|--------|------------|"

if [ -d "$REPO_ROOT/regions/system/server/rust" ]; then
  for dir in "$REPO_ROOT"/regions/system/server/rust/*/; do
    [ -d "$dir" ] || continue
    name=$(basename "$dir")
    maturity=$(check_maturity "$dir")
    key_files=$(check_key_files "$dir")
    echo "| $name | \`$maturity\` | $key_files |"

    case "$maturity" in
      production) count_production=$((count_production + 1)) ;;
      beta) count_beta=$((count_beta + 1)) ;;
      experimental) count_experimental=$((count_experimental + 1)) ;;
      *) count_template=$((count_template + 1)) ;;
    esac
  done
fi

echo ""

# --- System サーバー (Go) ---
echo "## Systemティア — サーバー（Go）"
echo ""
echo "| モジュール | 成熟度 | キーファイル |"
echo "|-----------|--------|------------|"

if [ -d "$REPO_ROOT/regions/system/server/go" ]; then
  for dir in "$REPO_ROOT"/regions/system/server/go/*/; do
    [ -d "$dir" ] || continue
    name=$(basename "$dir")
    maturity=$(check_maturity "$dir")
    key_files=$(check_key_files "$dir")
    echo "| $name | \`$maturity\` | $key_files |"

    case "$maturity" in
      production) count_production=$((count_production + 1)) ;;
      beta) count_beta=$((count_beta + 1)) ;;
      experimental) count_experimental=$((count_experimental + 1)) ;;
      *) count_template=$((count_template + 1)) ;;
    esac
  done
fi

echo ""

# --- System ライブラリ (各言語) ---
for lang in go rust typescript dart; do
  display_lang="$lang"
  case "$lang" in
    go) display_lang="Go" ;;
    rust) display_lang="Rust" ;;
    typescript) display_lang="TypeScript" ;;
    dart) display_lang="Dart" ;;
  esac

  echo "## Systemティア — ライブラリ（$display_lang）"
  echo ""
  echo "| モジュール | 成熟度 | キーファイル |"
  echo "|-----------|--------|------------|"

  lib_dir="$REPO_ROOT/regions/system/library/$lang"
  if [ -d "$lib_dir" ]; then
    for dir in "$lib_dir"/*/; do
      [ -d "$dir" ] || continue
      name=$(basename "$dir")
      maturity=$(check_maturity "$dir")
      key_files=$(check_key_files "$dir")
      echo "| $name | \`$maturity\` | $key_files |"

      case "$maturity" in
        production) count_production=$((count_production + 1)) ;;
        beta) count_beta=$((count_beta + 1)) ;;
        experimental) count_experimental=$((count_experimental + 1)) ;;
        *) count_template=$((count_template + 1)) ;;
      esac
    done
  fi
  echo ""
done

# --- Business ティア ---
echo "## Businessティア"
echo ""
echo "| モジュール | 成熟度 | キーファイル |"
echo "|-----------|--------|------------|"

if [ -d "$REPO_ROOT/regions/business" ]; then
  for domain_dir in "$REPO_ROOT"/regions/business/*/; do
    [ -d "$domain_dir" ] || continue
    domain=$(basename "$domain_dir")
    for kind_dir in "$domain_dir"*/; do
      [ -d "$kind_dir" ] || continue
      kind=$(basename "$kind_dir")
      for lang_dir in "$kind_dir"*/; do
        [ -d "$lang_dir" ] || continue
        for svc_dir in "$lang_dir"*/; do
          [ -d "$svc_dir" ] || continue
          name="$domain/$(basename "$svc_dir")"
          maturity=$(check_maturity "$svc_dir")
          key_files=$(check_key_files "$svc_dir")
          echo "| $name | \`$maturity\` | $key_files |"

          case "$maturity" in
            production) count_production=$((count_production + 1)) ;;
            beta) count_beta=$((count_beta + 1)) ;;
            experimental) count_experimental=$((count_experimental + 1)) ;;
            *) count_template=$((count_template + 1)) ;;
          esac
        done
      done
    done
  done
fi

echo ""

# --- Service ティア ---
echo "## Serviceティア"
echo ""
echo "| モジュール | 成熟度 | キーファイル |"
echo "|-----------|--------|------------|"

if [ -d "$REPO_ROOT/regions/service" ]; then
  for svc_dir in "$REPO_ROOT"/regions/service/*/; do
    [ -d "$svc_dir" ] || continue
    svc_name=$(basename "$svc_dir")
    for kind_dir in "$svc_dir"*/; do
      [ -d "$kind_dir" ] || continue
      for lang_dir in "$kind_dir"*/; do
        [ -d "$lang_dir" ] || continue
        for module_dir in "$lang_dir"*/; do
          [ -d "$module_dir" ] || continue
          name="$svc_name/$(basename "$module_dir")"
          maturity=$(check_maturity "$module_dir")
          key_files=$(check_key_files "$module_dir")
          echo "| $name | \`$maturity\` | $key_files |"

          case "$maturity" in
            production) count_production=$((count_production + 1)) ;;
            beta) count_beta=$((count_beta + 1)) ;;
            experimental) count_experimental=$((count_experimental + 1)) ;;
            *) count_template=$((count_template + 1)) ;;
          esac
        done
      done
    done
  done
fi

echo ""

# サマリー
total=$((count_production + count_beta + count_experimental + count_template))
echo "---"
echo ""
echo "## サマリー"
echo ""
echo "| レベル | モジュール数 | 割合 |"
echo "|--------|-------------|------|"
if [ "$total" -gt 0 ]; then
  echo "| \`production\` | $count_production | $((count_production * 100 / total))% |"
  echo "| \`beta\` | $count_beta | $((count_beta * 100 / total))% |"
  echo "| \`experimental\` | $count_experimental | $((count_experimental * 100 / total))% |"
  echo "| \`template-only\` | $count_template | $((count_template * 100 / total))% |"
  echo "| **合計** | **$total** | **100%** |"
else
  echo "| (モジュールなし) | 0 | - |"
fi
