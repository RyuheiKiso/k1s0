#!/usr/bin/env bash
# Go ソースコードの静的セキュリティ解析を gosec で実行する
# gosec は Go の一般的なセキュリティ脆弱性パターンを検出する SAST ツール
set -euo pipefail

echo "=== Go SAST (gosec) ==="
# list-modules.sh を使って modules.yaml から Go モジュールを取得（skip-ci 除外）
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
chmod +x "$SCRIPT_DIR/../list-modules.sh"
mapfile -t modules < <("$SCRIPT_DIR/../list-modules.sh" --lang go --no-skip-ci)
echo "Found ${#modules[@]} Go module(s)"

failed=0
for dir in "${modules[@]}"; do
    echo "--- Scanning $dir ---"
    # -fmt text: テーブル形式で出力
    # -severity medium: Medium 以上の問題を検出
    # -confidence medium: Medium 以上の確信度で報告
    # -exclude-generated: 生成コードを除外（proto 生成ファイル等）
    if ! (cd "$dir" && gosec -fmt text -severity medium -confidence medium -exclude-generated ./...); then
        failed=1
    fi
done

exit $failed
