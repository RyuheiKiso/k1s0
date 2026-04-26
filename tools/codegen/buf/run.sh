#!/usr/bin/env bash
#
# tools/codegen/buf/run.sh — buf generate を呼ぶ薄いラッパ
#
# 設計: docs/05_実装/20_コード生成設計/（IMP-CODEGEN-002 / 004）
# 関連: buf.gen.yaml（リポジトリルート）/ src/contracts/buf.yaml（workspace）
#
# 役割:
#   1. リポジトリルートに移動
#   2. buf dep update で buf.lock を最新化（オプション）
#   3. buf generate を src/contracts に対して実行
#   4. 生成先（src/sdk/{dotnet,go,rust,typescript}/generated/）に出力
#
# Usage:
#   tools/codegen/buf/run.sh                # 通常の生成
#   tools/codegen/buf/run.sh --update-deps  # buf.lock も更新
#   tools/codegen/buf/run.sh --check        # 生成物の差分を確認するだけ
#   tools/codegen/buf/run.sh --lint         # buf lint のみ実行
#   tools/codegen/buf/run.sh --breaking     # buf breaking を main に対して実行

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "${REPO_ROOT}"

UPDATE_DEPS=0
CHECK=0
LINT_ONLY=0
BREAKING_ONLY=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --update-deps) UPDATE_DEPS=1; shift ;;
        --check) CHECK=1; shift ;;
        --lint) LINT_ONLY=1; shift ;;
        --breaking) BREAKING_ONLY=1; shift ;;
        -h|--help)
            sed -n '3,21p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *)
            echo "[error] 未知のオプション: $1" >&2
            exit 1
            ;;
    esac
done

if ! command -v buf >/dev/null 2>&1; then
    echo "[error] buf CLI が見つかりません。Dev Container の docs-writer / sdk-dev / full プロファイルで導入されます。" >&2
    exit 1
fi

if [[ "${LINT_ONLY}" == "1" ]]; then
    echo "[info] buf lint"
    buf lint src/contracts
    exit 0
fi

if [[ "${BREAKING_ONLY}" == "1" ]]; then
    echo "[info] buf breaking --against .git#branch=main"
    buf breaking src/contracts --against ".git#branch=main"
    exit 0
fi

if [[ "${UPDATE_DEPS}" == "1" ]]; then
    echo "[info] buf dep update（src/contracts）"
    (cd src/contracts && buf dep update)
fi

echo "[info] buf lint（事前チェック）"
buf lint src/contracts

# tier1 module（公開 11 API + health + 共通型）→ 4 言語 SDK
# 引数なしで buf.gen.yaml の `inputs: directory` を活用（src/contracts/tier1 限定）
echo "[info] buf generate (tier1 → SDK 4 言語)"
buf generate --template buf.gen.yaml

# internal module（tier1 内部 gRPC）→ Go + Rust のみ（ADR-TIER1-003 言語不可視）
echo "[info] buf generate (internal → tier1 Go + Rust core のみ)"
buf generate --template buf.gen.internal.yaml

if [[ "${CHECK}" == "1" ]]; then
    echo "[info] 生成物の差分を確認"
    if ! git diff --exit-code -- src/sdk/*/generated; then
        echo "[error] 生成物が最新でありません。本スクリプトを再実行して git add してください。"
        exit 1
    fi
    echo "[ok] 生成物 diff なし"
fi

echo "[ok] codegen 完了"
