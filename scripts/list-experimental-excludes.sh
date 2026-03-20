#!/usr/bin/env bash
# experimental パッケージを --exclude フラグ付きで出力するスクリプト
# justfile の lint-rust / test-rust / build-rust で共通利用される
#
# 使い方: excludes=$(bash scripts/list-experimental-excludes.sh)
#   結果: "--exclude pkg1 --exclude pkg2 ..." が標準出力に出力される
#
# 注意: このファイルには実行権限が必要です (chmod +x scripts/list-experimental-excludes.sh)
set -euo pipefail

# modules.yaml から experimental Rust モジュールを取得し --exclude フラグに変換
excludes=""
while IFS= read -r dir; do
    # Cargo.toml から実際の package name を取得（basename と package name が異なる場合に対応）
    pkg_name=$(grep -m1 '^name' "$dir/Cargo.toml" | sed 's/.*"\(.*\)"/\1/')
    excludes="$excludes --exclude $pkg_name"
done < <(scripts/list-modules.sh --lang rust --status experimental)

# exclude 対象が workspace に存在するか検証（find で検索深度を制限）
ws_packages=$(find regions/system -maxdepth 4 -name 'Cargo.toml' -exec grep -h '^name' {} \; 2>/dev/null | sed 's/.*"\(.*\)"/\1/')
for exc in $excludes; do
    if [ "$exc" = "--exclude" ]; then continue; fi
    if ! echo "$ws_packages" | grep -qx "$exc"; then
        echo "ERROR: excluded package '$exc' not found in workspace" >&2
        exit 1
    fi
done

# 結果を標準出力に出力（呼び出し元でキャプチャする用途）
echo "$excludes"
