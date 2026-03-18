#!/usr/bin/env bash
# Rust ワークスペース・クレートの脆弱性監査を実行する
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

# business 層の Rust クレートを検索してスキャン
# 各サービスが独立した Cargo.toml を持つため、個別にロックファイルを生成して監査する
for cargo_toml in $(find regions/business -name "Cargo.toml" -path "*/rust/*/Cargo.toml" 2>/dev/null); do
    dir=$(dirname "$cargo_toml")
    echo "--- Scanning $dir ---"
    if ! (cd "$dir" && cargo generate-lockfile --quiet 2>/dev/null; cargo audit); then
        failed=1
    fi
done

# service 層の Rust クレートを検索してスキャン
# 各サービスが独立した Cargo.toml を持つため、個別にロックファイルを生成して監査する
for cargo_toml in $(find regions/service -name "Cargo.toml" -path "*/rust/*/Cargo.toml" 2>/dev/null); do
    dir=$(dirname "$cargo_toml")
    echo "--- Scanning $dir ---"
    if ! (cd "$dir" && cargo generate-lockfile --quiet 2>/dev/null; cargo audit); then
        failed=1
    fi
done

exit $failed
