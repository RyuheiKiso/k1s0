#!/usr/bin/env bash
#
# tools/lint/test-tag-lint.sh
#
# テスト属性タグ規約の機械検証（ADR-TEST-007）。
#
# 設計正典: ADR-TEST-007（テスト属性タグ + CI 実行フェーズ分離）
# 関連 ID: IMP-CI-TAG-001（4 タグ正典化）/ IMP-CI-TAG-003（言語別実装）
#
# 検証対象（リリース時点での最小成立形）:
#   1. tests/ 配下の Go test ファイルで `// +build` か `//go:build` の build tag
#      が記述されている場合、4 タグ最低セット（slow / flaky / security / nightly）の
#      いずれかに整合するかを確認する
#   2. build tag に typo（slowww / flakey 等）が無いか
#
# 採用初期で本格化する射程:
#   - 5 秒超の test に //go:build slow 必須化（go test -json で実測値取得）
#   - Rust ignore / xUnit Trait / Vitest filter の同等規約を 4 言語で機械検証
#   - PR レビュー時に本 lint を pre-push hook で強制
#
# Usage:
#   tools/lint/test-tag-lint.sh
#
# 終了コード:
#   0 = 全 build tag が 4 タグ最低セットと整合 / 1 = typo 検出 / 2 = 引数 / 環境エラー

set -euo pipefail

# 4 タグ最低セット（ADR-TEST-007 IMP-CI-TAG-001）
ALLOWED_TAGS=("slow" "flaky" "security" "nightly")

# repo root
REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$REPO_ROOT"

# 検査対象 ディレクトリ
TARGETS=(tests)

# 違反件数カウンタ
violations=0

# `//go:build <tag>` または `// +build <tag>` を含む Go ファイルを検査
for dir in "${TARGETS[@]}"; do
    if [[ ! -d "$dir" ]]; then
        continue
    fi
    while IFS= read -r file; do
        # build tag の行を抽出（最大 30 行目までに記述される慣習）
        tags=$(awk 'NR <= 30 && /^\/\/(go:build| \+build) /' "$file" || true)
        if [[ -z "$tags" ]]; then
            continue
        fi
        # 各 tag 行について 4 タグ最低セットとの照合
        while IFS= read -r tag_line; do
            # tag 部分を抽出（//go:build <expr> または // +build <expr>）
            expr=$(echo "$tag_line" | sed -E 's@^//(go:build| \+build)[[:space:]]+(.*)$@\2@')
            # 各 token を分割して識別子を取り出す（| && スペース , を区切り扱い）
            tokens=$(echo "$expr" | tr '|&,()! ' '\n' | grep -E '^[a-zA-Z][a-zA-Z0-9_]*$' || true)
            for token in $tokens; do
                # 4 タグ最低セットに含まれているか
                matched=0
                for allowed in "${ALLOWED_TAGS[@]}"; do
                    if [[ "$token" == "$allowed" ]]; then
                        matched=1
                        break
                    fi
                done
                if [[ "$matched" -eq 0 ]]; then
                    echo "[warn] $file: build tag '$token' は ADR-TEST-007 4 タグ最低セット（${ALLOWED_TAGS[*]}）に未登録"
                    echo "       tag を追加する場合は ADR-TEST-007 を改訂してから本 lint の ALLOWED_TAGS を更新してください"
                    violations=$((violations + 1))
                fi
            done
        done <<< "$tags"
    done < <(find "$dir" -type f -name "*_test.go")
done

# typo 検出: 4 タグの 1 文字 typo 候補（slowww / flakey 等）を grep で警告
declare -A TYPOS=(
    ["slowww"]="slow"
    ["sloww"]="slow"
    ["flakey"]="flaky"
    ["flackey"]="flaky"
    ["securty"]="security"
    ["securit"]="security"
    ["nigtly"]="nightly"
    ["nighlty"]="nightly"
)
for typo in "${!TYPOS[@]}"; do
    if grep -rn --include="*_test.go" -E "//(go:build| \+build).*\\b${typo}\\b" tests 2>/dev/null; then
        echo "[error] typo 検出: '${typo}' は '${TYPOS[$typo]}' の typo 可能性"
        violations=$((violations + 1))
    fi
done

# 結果報告
if [[ "$violations" -gt 0 ]]; then
    echo "[fail] ADR-TEST-007 build tag lint: ${violations} 件の違反"
    exit 1
fi

echo "[ok] ADR-TEST-007 build tag lint: 全 *_test.go が 4 タグ最低セットと整合"
exit 0
