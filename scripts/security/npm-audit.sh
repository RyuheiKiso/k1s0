#!/usr/bin/env bash
# npm パッケージの脆弱性監査を全プロジェクトで実行する
set -euo pipefail

echo "=== npm Audit ==="
# list-modules.sh を使って modules.yaml から TS モジュールを取得（skip-ci 除外）
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
chmod +x "$SCRIPT_DIR/../list-modules.sh"
mapfile -t packages < <("$SCRIPT_DIR/../list-modules.sh" --lang ts --no-skip-ci)
echo "Found ${#packages[@]} npm project(s)"

failed=0
for dir in "${packages[@]}"; do
    echo "--- Scanning $dir ---"
    if ! (cd "$dir" && npm audit --audit-level=high); then
        failed=1
    fi
done

exit $failed
