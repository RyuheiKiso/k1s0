#!/usr/bin/env bash
# 統合テスト対象サーバーを JSON 配列で出力する
# integration-test.yaml のマトリクスジョブで使用
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# integration_test.rs を持つサーバーディレクトリを検出
servers=()
for test_file in "$REPO_ROOT"/regions/system/server/rust/*/tests/integration_test.rs; do
  if [ -f "$test_file" ]; then
    server_dir="$(dirname "$(dirname "$test_file")")"
    servers+=("$(basename "$server_dir")")
  fi
done

# JSON 配列として出力
printf '%s\n' "${servers[@]}" | jq -R . | jq -s -c .
