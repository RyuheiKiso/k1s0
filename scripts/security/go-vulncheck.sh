#!/usr/bin/env bash
# Go モジュールの脆弱性スキャンを全モジュールで実行する
set -euo pipefail

echo "=== Go Vulnerability Check ==="
# list-modules.sh を使って modules.yaml から Go モジュールを取得（skip-ci 除外）
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
chmod +x "$SCRIPT_DIR/../list-modules.sh"
mapfile -t modules < <("$SCRIPT_DIR/../list-modules.sh" --lang go --no-skip-ci)
echo "Found ${#modules[@]} Go module(s)"

failed=0
for dir in "${modules[@]}"; do
    echo "--- Scanning $dir ---"
    if ! (cd "$dir" && govulncheck ./...); then
        failed=1
    fi
done

exit $failed
