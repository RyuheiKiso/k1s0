#!/usr/bin/env bash
# 統合テスト対象サーバーを JSON 配列で出力する
# integration-test.yaml のマトリクスジョブで使用
# list-modules.sh を再利用し、stable かつ rust かつ server のパスを取得後、
# integration_test.rs が存在するもののみをフィルタする
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# list-modules.sh で stable/rust/server モジュールのパスを取得し、
# system tier のみに絞る（CI workflow は regions/system workspace で実行するため）
# grep マッチ0件時の exit code 1 で set -e 異常終了を防ぐため || true を付与
paths=$("$SCRIPT_DIR/list-modules.sh" --lang rust --status stable --type server | grep "^regions/system/" || true)

# integration_test.rs を持つサーバーのみ抽出し、basename を収集
servers=()
while IFS= read -r mod_path; do
  # 空行をスキップ
  [ -z "$mod_path" ] && continue
  # integration_test.rs が存在するか確認
  if [ -f "$REPO_ROOT/$mod_path/tests/integration_test.rs" ]; then
    servers+=("$(basename "$mod_path")")
  fi
done <<< "$paths"

# JSON 配列として出力
if [ ${#servers[@]} -eq 0 ]; then
  echo "[]"
else
  printf '%s\n' "${servers[@]}" | jq -R . | jq -s -c .
fi
