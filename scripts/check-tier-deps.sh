#!/usr/bin/env bash
# エラー発生時に即座に終了し、未定義変数をエラーとして扱い、パイプラインの途中エラーも検知する（M-26対応）
set -euo pipefail
# Tier 依存関係の整合性チェック。
# modules.yaml に定義された tier_dependencies に基づき、
# 上位層（service/business）から下位層（system）への依存のみを許可する。
# 逆方向の依存（system → business, system → service, business → service）を検出してエラーにする。
#
# 使い方:
#   bash scripts/check-tier-deps.sh
#
# 終了コード:
#   0: 問題なし
#   1: 不正な tier 間依存を検出

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "=== Tier 依存関係チェック ==="

failed=0

# system 層のサービスが business/service 層に依存していないかチェック（Cargo.toml）
echo "--- system tier: business/service への逆依存チェック ---"
while IFS= read -r cargo_toml; do
    dir="$(dirname "$cargo_toml")"
    # system 層ディレクトリかどうか確認
    if echo "$dir" | grep -qE 'regions/system/(server|library)'; then
        # [dependencies] と [build-dependencies] セクションのみを対象にパターンを検索する。
        # [dev-dependencies] は除外し、テスト専用クレートによる偽陽性を防ぐ。
        # package の name フィールドがパターンに一致しても偽陽性にならないよう制限する。
        if sed -n '/^\[dependencies\]\|^\[build-dependencies\]/,/^\[/p' "$cargo_toml" 2>/dev/null | grep -qE 'k1s0-(business|service)-'; then
            echo "ERROR: system tier が business/service クレートに依存しています: $cargo_toml"
            sed -n '/^\[dependencies\]\|^\[build-dependencies\]/,/^\[/p' "$cargo_toml" 2>/dev/null | grep -E 'k1s0-(business|service)-'
            failed=1
        fi
    fi
done < <(find "${REPO_ROOT}/regions/system" -name "Cargo.toml" -not -path "*/target/*")

# system 層の Go モジュールが business/service モジュールに依存していないかチェック（go.mod）
while IFS= read -r go_mod; do
    dir="$(dirname "$go_mod")"
    if echo "$dir" | grep -qE 'regions/system/(server|library)'; then
        if grep -qE 'k1s0-platform/(business|service)-' "$go_mod" 2>/dev/null; then
            echo "ERROR: system tier が business/service モジュールに依存しています: $go_mod"
            grep -E 'k1s0-platform/(business|service)-' "$go_mod"
            failed=1
        fi
    fi
done < <(find "${REPO_ROOT}/regions/system" -name "go.mod" -not -path "*/vendor/*")

# business 層のサービスが service 層に依存していないかチェック（Cargo.toml）
echo "--- business tier: service への逆依存チェック ---"
while IFS= read -r cargo_toml; do
    dir="$(dirname "$cargo_toml")"
    if echo "$dir" | grep -qE 'regions/business'; then
        # [dependencies] と [build-dependencies] セクションのみを対象にパターンを検索する。
        # [dev-dependencies] は除外し、テスト専用クレートによる偽陽性を防ぐ。
        # package の name フィールドがパターンに一致しても偽陽性にならないよう制限する。
        if sed -n '/^\[dependencies\]\|^\[build-dependencies\]/,/^\[/p' "$cargo_toml" 2>/dev/null | grep -qE 'k1s0-service-'; then
            echo "ERROR: business tier が service クレートに依存しています: $cargo_toml"
            sed -n '/^\[dependencies\]\|^\[build-dependencies\]/,/^\[/p' "$cargo_toml" 2>/dev/null | grep -E 'k1s0-service-'
            failed=1
        fi
    fi
done < <(find "${REPO_ROOT}/regions/business" -name "Cargo.toml" -not -path "*/target/*" 2>/dev/null || true)

if [ "$failed" -eq 0 ]; then
    echo "OK: 不正な tier 間依存は検出されませんでした"
else
    echo "FAIL: 不正な tier 間依存が検出されました"
fi

exit $failed
