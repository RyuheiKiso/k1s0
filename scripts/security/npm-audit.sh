#!/usr/bin/env bash
# npm パッケージの脆弱性監査を全プロジェクトで実行する
set -euo pipefail

echo "=== npm Audit ==="
mapfile -t lockfiles < <(rg --files -g 'package-lock.json' regions CLI | sort)
echo "Found ${#lockfiles[@]} npm project(s)"

failed=0
for lockfile in "${lockfiles[@]}"; do
    dir="$(dirname "$lockfile")"
    echo "--- Scanning $dir ---"
    if ! (cd "$dir" && npm audit --audit-level=high); then
        failed=1
    fi
done

exit $failed
