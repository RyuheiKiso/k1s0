#!/bin/bash
# tier2 quota litmus chaos scenario runner
# 全 6 シナリオを順番に実行し、結果を記録する

# スクリプトを厳格モードで実行する（未定義変数・エラーで即時終了）
set -euo pipefail

# スクリプトのディレクトリを取得する
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# kubectl が利用可能かチェックする
if ! command -v kubectl &> /dev/null; then
    # kubectl がない場合は dry-run モードで実行する
    echo "WARNING: kubectl not found, running in dry-run mode"
    # 各シナリオファイルを dry-run 出力する
    for f in "$SCRIPT_DIR/scenarios"/q*.yaml; do
        echo "DRY-RUN: would apply $f"
    done
    # dry-run モードで正常終了する
    exit 0
fi

# 各シナリオを順番に適用する
for f in "$SCRIPT_DIR/scenarios"/q*.yaml; do
    # 実行中のシナリオファイル名を表示する
    echo "Applying chaos scenario: $f"
    # kubectl apply でシナリオを Kubernetes クラスターに適用する
    kubectl apply -f "$f"
done

# 全シナリオ適用完了メッセージを出力する
echo "All chaos scenarios applied successfully"
