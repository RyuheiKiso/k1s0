#!/usr/bin/env bash
# CICD-01 監査対応: paths-ignore の手動管理リスクを機械的に検証するスクリプト。
# 各サービス固有 CI (*-ci.yaml) の paths: セクションと ci.yaml の paths-ignore: の差異を検出する。
#
# 使用方法:
#   bash scripts/sync-paths-ignore.sh [--fix]
#
# オプション:
#   --fix   差異を自動修正する（ci.yaml の paths-ignore を更新）
#   (なし)  差異をチェックのみして終了コードで報告する（CI/CD 用）
#
# 終了コード:
#   0: 差異なし（または --fix で修正完了）
#   1: 差異あり（--fix なし時）

set -euo pipefail

# リポジトリルートを特定する
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CI_YAML="$REPO_ROOT/.github/workflows/ci.yaml"
WORKFLOWS_DIR="$REPO_ROOT/.github/workflows"
FIX_MODE=false

# --fix フラグを解析する
for arg in "$@"; do
    if [ "$arg" = "--fix" ]; then
        FIX_MODE=true
    fi
done

echo "=== CICD-01: paths-ignore 整合性チェック ==="
echo "対象: $CI_YAML"
echo ""

# -------------------------------------------------------------------
# Step 1: サービス固有 CI (*-ci.yaml) の paths: セクションを収集する
# -------------------------------------------------------------------
# pull_request の paths: ブロックに定義されたトリガーパスを全ファイルから収集する
SERVICE_PATHS=$(
    for workflow in "$WORKFLOWS_DIR"/*-ci.yaml; do
        [ -f "$workflow" ] || continue
        awk '/^  pull_request:/{found=1} found && /^    paths:/{in_paths=1; next} in_paths && /^      - /{gsub(/^      - |'"'"'| *$/, ""); print} in_paths && /^    [a-z]/{exit} in_paths && /^  [a-z]/{exit}' "$workflow" 2>/dev/null || true
    done | grep -v "^$" | sort | uniq
)

# -------------------------------------------------------------------
# Step 2: ci.yaml の paths-ignore エントリを収集する
# -------------------------------------------------------------------
# paths-ignore ブロックの各エントリをシングルクォートを除去して取得する
IGNORE_PATHS=$(
    awk '/^    paths-ignore:/{found=1; next} found && /^      - /{gsub(/^      - |'"'"'| *$/, ""); print} found && /^  [a-z]/{exit}' "$CI_YAML" 2>/dev/null \
    | grep -v "^#" \
    | grep -v "^$" \
    | sort
)

# -------------------------------------------------------------------
# Step 3: 差異を検出する（サービス CI にあって paths-ignore にないエントリ）
# -------------------------------------------------------------------
MISSING_IN_IGNORE=""
while IFS= read -r sp; do
    [ -z "$sp" ] && continue
    if ! echo "$IGNORE_PATHS" | grep -qF "$sp"; then
        MISSING_IN_IGNORE="${MISSING_IN_IGNORE}${sp}"$'\n'
    fi
done <<< "$SERVICE_PATHS"

# 末尾の改行を除去する
MISSING_IN_IGNORE="${MISSING_IN_IGNORE%$'\n'}"

# -------------------------------------------------------------------
# Step 4: 結果を報告する
# -------------------------------------------------------------------
if [ -z "$MISSING_IN_IGNORE" ]; then
    echo "✓ 差異なし: paths-ignore は最新です"
    exit 0
fi

echo "⚠ 差異検出: ci.yaml の paths-ignore に以下が不足しています:"
while IFS= read -r p; do
    [ -z "$p" ] && continue
    echo "  - $p"
done <<< "$MISSING_IN_IGNORE"

if $FIX_MODE; then
    echo ""
    echo "→ --fix モード: ci.yaml を自動更新します"
    # paths-ignore ブロックの末尾エントリの直後に不足エントリを追加する
    # awk を使って paths-ignore ブロックを検出し、ブロック終了直前に行を挿入する
    ENTRIES_TO_ADD=""
    while IFS= read -r p; do
        [ -z "$p" ] && continue
        ENTRIES_TO_ADD="${ENTRIES_TO_ADD}      - '${p}'"$'\n'
    done <<< "$MISSING_IN_IGNORE"

    # 一時ファイルを使って awk で安全に挿入する
    TMPFILE=$(mktemp)
    awk -v new_entries="$ENTRIES_TO_ADD" '
        /^    paths-ignore:/{in_ignore=1; print; next}
        in_ignore && /^      - /{print; next}
        in_ignore && !/^      - /{
            # paths-ignore ブロック終了直前に不足エントリを挿入する
            printf "%s", new_entries
            in_ignore=0
            print
            next
        }
        {print}
    ' "$CI_YAML" > "$TMPFILE"
    mv "$TMPFILE" "$CI_YAML"
    echo "✓ 更新完了: $(echo "$MISSING_IN_IGNORE" | grep -c "^[^$]") 件のエントリを追加しました"
    exit 0
else
    echo ""
    echo "修正するには: bash scripts/sync-paths-ignore.sh --fix"
    exit 1
fi
