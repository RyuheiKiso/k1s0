#!/usr/bin/env bash
# 統合テスト対象サーバーを JSON 配列で出力する
# integration-test.yaml のマトリクスジョブで使用
# --tier オプションで対象ティアを指定できる（デフォルト: system）
set -euo pipefail

# jq の存在チェック（JSON 配列出力に必要）
if ! command -v jq >/dev/null 2>&1; then
  echo "::error::jq が見つかりません。'apt-get install jq' または 'brew install jq' を実行してください。" >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# ティア引数のパース（デフォルト: system）
TIER="${1:-system}"

# integration_test.rs を持つサーバーのみ抽出し、basename を収集
servers=()

case "$TIER" in
  system)
    # list-modules.sh で stable/rust/server モジュールのパスを取得し、
    # system tier のみに絞る（CI workflow は regions/system workspace で実行するため）
    # grep マッチ0件時の exit code 1 で set -e 異常終了を防ぐため || true を付与
    paths=$("$SCRIPT_DIR/list-modules.sh" --lang rust --status stable --type server | grep "^regions/system/" || true)
    while IFS= read -r mod_path; do
      [ -z "$mod_path" ] && continue
      if [ -f "$REPO_ROOT/$mod_path/tests/integration_test.rs" ]; then
        servers+=("$(basename "$mod_path")")
      fi
    done <<< "$paths"
    ;;

  business)
    # business tier: regions/business/{domain}/server/rust/{service} を検索する
    for cargo_toml in "$REPO_ROOT"/regions/business/*/server/rust/*/Cargo.toml; do
      [ -f "$cargo_toml" ] || continue
      server_dir="$(dirname "$cargo_toml")"
      if [ -f "$server_dir/tests/integration_test.rs" ]; then
        servers+=("$(basename "$server_dir")")
      fi
    done
    ;;

  service)
    # service tier: regions/service/{service}/server/rust/{service} を検索する
    for cargo_toml in "$REPO_ROOT"/regions/service/*/server/rust/*/Cargo.toml; do
      [ -f "$cargo_toml" ] || continue
      server_dir="$(dirname "$cargo_toml")"
      if [ -f "$server_dir/tests/integration_test.rs" ]; then
        servers+=("$(basename "$server_dir")")
      fi
    done
    ;;

  *)
    echo "Usage: $(basename "$0") [system|business|service]" >&2
    exit 1
    ;;
esac

# JSON 配列として出力
if [ ${#servers[@]} -eq 0 ]; then
  echo "[]"
else
  printf '%s\n' "${servers[@]}" | jq -R . | jq -s -c .
fi
