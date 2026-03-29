#!/usr/bin/env bash
# Rust ワークスペースの脆弱性監査を実行する
# ハードコードされたパスの代わりに modules.yaml の type: workspace エントリを参照する
set -euo pipefail

echo "=== Cargo Audit ==="
failed=0

# スクリプトの場所からリポジトリルートを特定する
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# modules.yaml から lang: rust かつ type: workspace のパスを取得し、
# Cargo.lock が存在するワークスペースのみを監査対象とする
while IFS= read -r workspace_path; do
    if [ -f "$REPO_ROOT/$workspace_path/Cargo.lock" ]; then
        echo "--- Scanning $workspace_path/ ---"
        # deny.toml の ignore リストと同期して既知の受け入れ済み Advisory を除外する。
        # RUSTSEC-2023-0071: sqlx-mysql 経由の間接依存（rsa 0.9.x Marvin Attack、上流修正待ち）
        # RUSTSEC-2025-0111: testcontainers 経由の間接依存（テスト専用、本番影響なし）
        if ! (cd "$REPO_ROOT/$workspace_path" && cargo audit \
            --ignore RUSTSEC-2023-0071 \
            --ignore RUSTSEC-2025-0111); then
            failed=1
        fi
    fi
done < <("$SCRIPT_DIR/../list-modules.sh" --lang rust --type workspace)

exit $failed
