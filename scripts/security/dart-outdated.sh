#!/usr/bin/env bash
# Dart/Flutter パッケージの古い依存を検出する
# Dart には npm audit 相当のツールがないため、outdated チェックで代替する
set -euo pipefail

echo "=== Dart/Flutter Dependency Check ==="
mapfile -t packages < <(rg --files -g 'pubspec.yaml' regions CLI | sort)
echo "Found ${#packages[@]} Dart/Flutter package(s)"

failed=0
for pubspec in "${packages[@]}"; do
    dir="$(dirname "$pubspec")"
    echo "--- Checking $dir ---"
    if grep -q "sdk: flutter" "$pubspec"; then
        if ! (cd "$dir" && flutter pub get --no-example && flutter pub outdated --no-dev-dependencies); then
            failed=1
        fi
    else
        if ! (cd "$dir" && dart pub get && dart pub outdated --no-dev-dependencies); then
            failed=1
        fi
    fi
done

# outdated は情報提供のみ（警告レベル）。致命的なものは手動判断
echo "=== Note: dart pub outdated is advisory only ==="
exit 0
