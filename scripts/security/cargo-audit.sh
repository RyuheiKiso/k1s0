#!/usr/bin/env bash
# Rust ワークスペースの脆弱性監査を実行する
set -euo pipefail

echo "=== Cargo Audit ==="
failed=0

# CLI ワークスペース
if [ -f "CLI/Cargo.lock" ]; then
    echo "--- Scanning CLI/ ---"
    if ! (cd CLI && cargo audit); then
        failed=1
    fi
fi

# regions/system ワークスペース
if [ -f "regions/system/Cargo.lock" ]; then
    echo "--- Scanning regions/system/ ---"
    if ! (cd regions/system && cargo audit); then
        failed=1
    fi
fi

exit $failed
