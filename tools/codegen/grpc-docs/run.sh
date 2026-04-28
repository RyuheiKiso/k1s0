#!/usr/bin/env bash
#
# tools/codegen/grpc-docs/run.sh — proto → gRPC reference docs (Markdown)
#
# 設計: plan/03_Contracts実装/07_gRPC_reference_docs.md
# 関連 ID: IMP-CODEGEN-007（gRPC reference docs）/ ADR-BS-001
#
# 出力:
#   docs/02_構想設計/02_tier1設計/grpc-reference/v1/k1s0-tier1-grpc.md
#
# Usage:
#   tools/codegen/grpc-docs/run.sh         # 通常生成
#   tools/codegen/grpc-docs/run.sh --check # diff 検出のみ（CI 用）

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "${REPO_ROOT}"

CHECK=0
for arg in "$@"; do
    case "$arg" in
        --check) CHECK=1 ;;
        -h|--help)
            sed -n '3,15p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *)
            echo "[error] 未知の引数: $arg" >&2
            exit 1
            ;;
    esac
done

if ! command -v buf >/dev/null 2>&1; then
    echo "[error] buf CLI が見つかりません" >&2
    exit 1
fi

mkdir -p docs/02_構想設計/02_tier1設計/grpc-reference/v1

echo "[info] buf generate (gRPC reference docs)"
buf generate --template buf.gen.docs.yaml

out_file="docs/02_構想設計/02_tier1設計/grpc-reference/v1/k1s0-tier1-grpc.md"
if [[ -f "${out_file}" ]]; then
    lines=$(wc -l < "${out_file}")
    echo "[ok] 出力: ${out_file} (${lines} 行)"
else
    echo "[warn] 期待出力 ${out_file} が見つかりません"
    find docs/02_構想設計/02_tier1設計/grpc-reference -type f 2>/dev/null
fi

if [[ "${CHECK}" == "1" ]]; then
    target='docs/02_構想設計/02_tier1設計/grpc-reference'
    # 1) 既追跡ファイルの変更検出
    if ! git diff --exit-code -- "${target}"; then
        echo "[error] gRPC reference docs が最新でありません。"
        echo "  対処: tools/codegen/grpc-docs/run.sh を再実行し、git add してください。"
        exit 1
    fi
    # 2) untracked（新規生成）も検出。proto 追加で reference doc が増えた時の取りこぼし防止。
    untracked=$(git ls-files --others --exclude-standard -- "${target}")
    if [[ -n "${untracked}" ]]; then
        echo "[error] gRPC reference docs に未追跡ファイルがあります。git add してください:" >&2
        echo "${untracked}" >&2
        exit 1
    fi
    echo "[ok] gRPC docs diff なし"
fi
