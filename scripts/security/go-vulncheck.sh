#!/usr/bin/env bash
# Go モジュールの脆弱性スキャンを全モジュールで実行する
set -euo pipefail

echo "=== Go Vulnerability Check ==="
mapfile -t modules < <(rg --files -g 'go.mod' regions CLI | sort)
echo "Found ${#modules[@]} Go module(s)"

failed=0
for mod in "${modules[@]}"; do
    dir="$(dirname "$mod")"
    echo "--- Scanning $dir ---"
    if ! (cd "$dir" && govulncheck ./...); then
        failed=1
    fi
done

exit $failed
