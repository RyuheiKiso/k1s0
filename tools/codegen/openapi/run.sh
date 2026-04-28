#!/usr/bin/env bash
#
# tools/codegen/openapi/run.sh — proto → OpenAPI v2 (Swagger) export
#
# 設計: plan/03_Contracts実装/06_OpenAPI_export.md
# 関連 ID: IMP-CODEGEN-006（OpenAPI export）/ ADR-BS-001（Backstage 連携）
#
# 出力:
#   docs/02_構想設計/02_tier1設計/openapi/v1/k1s0-tier1.swagger.yaml
#   （allow_merge=true で 12 API + health + 共通型を 1 yaml にマージ）
#
# Usage:
#   tools/codegen/openapi/run.sh         # 通常生成
#   tools/codegen/openapi/run.sh --check # diff 検出のみ（CI 用）

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

# 出力ディレクトリ準備
mkdir -p docs/02_構想設計/02_tier1設計/openapi/v1

echo "[info] buf generate (OpenAPI v2)"
buf generate --template buf.gen.openapi.yaml

# 出力ファイルの確認
out_file="docs/02_構想設計/02_tier1設計/openapi/v1/k1s0-tier1.swagger.yaml"
if [[ -f "${out_file}" ]]; then
    lines=$(wc -l < "${out_file}")
    echo "[ok] 出力: ${out_file} (${lines} 行)"
else
    echo "[warn] 期待出力 ${out_file} が見つかりません"
    find docs/02_構想設計/02_tier1設計/openapi/v1 -name "*.yaml" -o -name "*.yml" -o -name "*.json" 2>/dev/null
fi

if [[ "${CHECK}" == "1" ]]; then
    target='docs/02_構想設計/02_tier1設計/openapi'
    # 1) 既追跡ファイルの変更検出
    if ! git diff --exit-code -- "${target}"; then
        echo "[error] OpenAPI が最新でありません。"
        echo "  対処: tools/codegen/openapi/run.sh を再実行し、git add してください。"
        exit 1
    fi
    # 2) untracked（新規生成）も検出。proto 追加で OpenAPI が増えた時の取りこぼし防止。
    untracked=$(git ls-files --others --exclude-standard -- "${target}")
    if [[ -n "${untracked}" ]]; then
        echo "[error] OpenAPI に未追跡ファイルがあります。git add してください:" >&2
        echo "${untracked}" >&2
        exit 1
    fi
    echo "[ok] OpenAPI diff なし"
fi
