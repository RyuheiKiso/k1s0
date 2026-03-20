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
        if ! (cd "$REPO_ROOT/$workspace_path" && cargo audit); then
            failed=1
        fi
    fi
done < <("$SCRIPT_DIR/../list-modules.sh" --lang rust --type workspace)

exit $failed
