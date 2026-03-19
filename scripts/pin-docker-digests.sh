#!/usr/bin/env bash
# =====================================================================
# ベースイメージのSHA256ダイジェスト固定スクリプト
#
# 全DockerfileのFROM行を走査し、タグ指定のイメージをダイジェスト付きに置換する。
# ビルドステージ間の参照（FROM chef AS planner 等）はスキップする。
#
# 使い方:
#   ./scripts/pin-docker-digests.sh          # 全Dockerfileを更新
#   ./scripts/pin-docker-digests.sh --dry-run # 変更内容をプレビュー（ファイル変更なし）
#
# 前提条件:
#   - Docker CLIが利用可能であること（docker pull / docker inspect を使用）
#   - インターネット接続があること（イメージのpullが必要）
#
# CI/CDでの定期実行を推奨（例: 週次cronジョブ）
# =====================================================================
set -euo pipefail

# リポジトリルートに移動する
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

# ドライランモードの判定
DRY_RUN=false
if [[ "${1:-}" == "--dry-run" ]]; then
    DRY_RUN=true
    echo "[DRY-RUN] ファイルの変更は行いません"
fi

# ビルドステージのエイリアス名（外部イメージではないためスキップする）
STAGE_ALIASES="chef planner cook builder build"

# FROM行が外部イメージ参照かどうかを判定する関数
is_external_image() {
    local image_base="$1"
    for alias in $STAGE_ALIASES; do
        if [[ "$image_base" == "$alias" ]]; then
            return 1
        fi
    done
    return 0
}

# FROM行からイメージ名を抽出する関数（--platform= を除去）
extract_image() {
    local from_line="$1"
    # FROM [--platform=xxx] image [AS name] の形式からイメージを取得する
    local image
    image=$(echo "$from_line" | sed -E 's/^FROM\s+//' | sed -E 's/--platform=[^ ]+\s+//' | awk '{print $1}')
    echo "$image"
}

# イメージのダイジェストを取得する関数
get_digest() {
    local image="$1"

    # テンプレート変数（{{ ... }}）を含む場合はスキップする
    if echo "$image" | grep -q '{{'; then
        echo ""
        return
    fi

    # 既にダイジェスト指定の場合はスキップする
    if echo "$image" | grep -q '@sha256:'; then
        echo ""
        return
    fi

    # docker pullでイメージを取得し、ダイジェストを取得する
    echo "  Pulling $image ..." >&2
    if ! docker pull "$image" > /dev/null 2>&1; then
        echo "  [WARNING] $image のpullに失敗しました。スキップします。" >&2
        echo ""
        return
    fi

    # ダイジェスト付きのフルイメージ名を取得する
    local digested
    digested=$(docker inspect --format='{{index .RepoDigests 0}}' "$image" 2>/dev/null || true)

    if [[ -z "$digested" ]]; then
        echo "  [WARNING] $image のダイジェスト取得に失敗しました。スキップします。" >&2
        echo ""
        return
    fi

    echo "$digested"
}

# Dockerfileの検索（target/ディレクトリを除外する）
DOCKERFILES=$(find . \( -name "Dockerfile" -o -name "Dockerfile.*" -o -name "*.Dockerfile" \) \
    -not -path "*/target/*" \
    -not -path "*/node_modules/*" \
    | sort)

# .teraテンプレートファイルも検索する（FROM行を含むもの）
TERA_FILES=$(find . -name "*.tera" -not -path "*/target/*" | sort)
for tera in $TERA_FILES; do
    if grep -q '^FROM ' "$tera" 2>/dev/null; then
        DOCKERFILES="$DOCKERFILES"$'\n'"$tera"
    fi
done

# 更新カウンター
UPDATED=0
SKIPPED=0
FAILED=0

echo "=================================================="
echo "Docker ベースイメージ ダイジェスト固定"
echo "=================================================="
echo ""

for dockerfile in $DOCKERFILES; do
    echo "Processing: $dockerfile"

    # FROM行を処理する
    while IFS= read -r line; do
        # FROM行からイメージ名を抽出する
        image=$(extract_image "$line")
        image_base="${image%%:*}"
        image_base="${image_base%%@*}"

        # ステージエイリアスはスキップする
        if ! is_external_image "$image_base"; then
            continue
        fi

        # テンプレート変数を含む場合はスキップする
        if echo "$image" | grep -q '{{'; then
            echo "  [SKIP] $image (テンプレート変数を含む)"
            ((SKIPPED++)) || true
            continue
        fi

        # 既にダイジェスト固定済みの場合はスキップする
        if echo "$image" | grep -q '@sha256:'; then
            echo "  [OK] $image (ダイジェスト固定済み)"
            ((SKIPPED++)) || true
            continue
        fi

        # ダイジェストを取得する
        digested=$(get_digest "$image")

        if [[ -z "$digested" ]]; then
            echo "  [FAIL] $image のダイジェスト取得に失敗"
            ((FAILED++)) || true
            continue
        fi

        echo "  [PIN] $image -> $digested"

        if [[ "$DRY_RUN" == "false" ]]; then
            # Dockerfile内のイメージ参照をダイジェスト付きに置換する
            # タグとダイジェストの両方を保持する形式: image:tag@sha256:...
            # sed でエスケープが必要な文字を処理する
            escaped_image=$(echo "$image" | sed 's/[\/&]/\\&/g')
            escaped_digested=$(echo "$digested" | sed 's/[\/&]/\\&/g')
            sed -i "s|${escaped_image}|${escaped_digested}|g" "$dockerfile"
        fi

        ((UPDATED++)) || true

    done < <(grep '^FROM ' "$dockerfile" 2>/dev/null || true)

    echo ""
done

echo "=================================================="
echo "完了サマリー"
echo "=================================================="
echo "  更新: $UPDATED 件"
echo "  スキップ: $SKIPPED 件"
echo "  失敗: $FAILED 件"

if [[ "$DRY_RUN" == "true" ]]; then
    echo ""
    echo "[DRY-RUN] 実際にファイルを更新するには --dry-run を外して実行してください"
fi
