#!/usr/bin/env bash
# Rust ソースコードの静的セキュリティ解析を clippy security lints で実行する
# clippy の security 関連 lint グループでセキュリティ問題を検出する
set -euo pipefail

echo "=== Rust SAST (clippy security lints) ==="
# list-modules.sh を使って modules.yaml から Rust モジュールを取得（skip-ci 除外）
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
chmod +x "$SCRIPT_DIR/../list-modules.sh"
mapfile -t modules < <("$SCRIPT_DIR/../list-modules.sh" --lang rust --no-skip-ci)
echo "Found ${#modules[@]} Rust module(s)"

failed=0
for dir in "${modules[@]}"; do
    echo "--- Scanning $dir ---"
    # clippy のセキュリティ関連 lint を有効化して検査する
    # -D warnings: 警告をエラーとして扱い CI を失敗させる
    # suspicious: 怪しいコードパターン（整数オーバーフロー等）
    # correctness: 誤りのあるコードパターン（パニックを誘発する可能性があるもの等）
    if ! (cd "$dir" && cargo clippy --all-targets --all-features -- \
        -D warnings \
        -W clippy::suspicious \
        -W clippy::correctness \
        -W clippy::integer_arithmetic \
        -A clippy::all 2>&1); then
        failed=1
    fi
done

exit $failed
