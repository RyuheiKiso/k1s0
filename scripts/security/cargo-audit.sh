#!/usr/bin/env bash
# Rust ワークスペースの脆弱性監査を実行する
# ハードコードされたパスの代わりに modules.yaml の type: workspace エントリを参照する
set -euo pipefail

echo "=== Cargo Audit ==="
failed=0

# スクリプトの場所からリポジトリルートを特定する
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# HIGH-005 監査対応: advisory-ignore-list.txt から --ignore フラグを動的に生成する
# これにより cargo-audit.sh と deny.toml の二重管理を解消し、
# advisory-ignore-list.txt を唯一の管理場所とする。
IGNORE_LIST="${SCRIPT_DIR}/advisory-ignore-list.txt"
IGNORE_FLAGS=""
while IFS= read -r line; do
    # コメント行（# で始まる行）と空行をスキップ
    [[ "$line" =~ ^[[:space:]]*(#|$) ]] && continue
    # 行の最初のフィールドがアドバイザリ ID
    advisory=$(echo "$line" | awk '{print $1}')
    [[ -n "$advisory" ]] && IGNORE_FLAGS="${IGNORE_FLAGS} --ignore ${advisory}"
done < "${IGNORE_LIST}"

# modules.yaml から lang: rust かつ type: workspace のパスを取得し、
# Cargo.lock が存在するワークスペースのみを監査対象とする
while IFS= read -r workspace_path; do
    if [ -f "$REPO_ROOT/$workspace_path/Cargo.lock" ]; then
        echo "--- Scanning $workspace_path/ ---"
        # shellcheck disable=SC2086
        if ! (cd "$REPO_ROOT/$workspace_path" && cargo audit ${IGNORE_FLAGS}); then
            failed=1
        fi
    fi
done < <("$SCRIPT_DIR/../list-modules.sh" --lang rust --type workspace)

exit $failed
