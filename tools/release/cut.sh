#!/usr/bin/env bash
#
# tools/release/cut.sh — release tag を切る唯一の正規入口
#
# 設計正典:
#   ADR-TEST-001（release tag 強制 3 重防御）
#   ADR-TEST-007（4 段フェーズ: PR / nightly / weekly / release tag）
# 関連 ID: IMP-CI-TAG-001〜005
#
# Usage:
#   tools/release/cut.sh <version>           # 例: tools/release/cut.sh v0.1.0
#   tools/release/cut.sh --dry-run <version> # qualify-release を実行せず tag のみ確認
#
# 強制ステップ:
#   1. git status クリーン確認（dirty なら exit 2）
#   2. 渡された version が SemVer 形式か確認
#   3. tag が既存でないか確認
#   4. make qualify-release 強制実行（L0–L5 + L7 + L9 + L10 全層）
#   5. qualify report を tar.zst 化
#   6. tag メッセージに qualify report の sha256 を埋め込む
#   7. git tag -a <version> -m "qualify: <sha256>"
#   8. push は手動（cut.sh は tag 作成までで止め、push は別操作）
#
# 設計理由:
#   ADR-TEST-001 で「release tag を切る行為そのものに qualify 強制を物理的に紐付ける」
#   を決定。git tag 直叩きを止めて本 wrapper のみを正規入口とすることで、CI なし環境でも
#   qualify 走行なしの release tag が原理的に切れない構造にする。
#
# 終了コード:
#   0 = tag 成功 / 1 = qualify 失敗 / 2 = 引数 / 環境エラー

set -euo pipefail

usage() {
    sed -n '2,30p' "$0" | sed 's/^# \{0,1\}//'
}

# 引数解析
DRY_RUN=0
VERSION=""
for arg in "$@"; do
    case "$arg" in
        --dry-run) DRY_RUN=1 ;;
        -h|--help) usage; exit 0 ;;
        v*.*.*) VERSION="$arg" ;;
        *)
            if [[ -z "$VERSION" && "$arg" =~ ^v[0-9]+\.[0-9]+\.[0-9]+ ]]; then
                VERSION="$arg"
            else
                echo "[error] unknown arg or invalid version: $arg" >&2
                usage
                exit 2
            fi
            ;;
    esac
done

if [[ -z "$VERSION" ]]; then
    echo "[error] version 必須（例: tools/release/cut.sh v0.1.0）" >&2
    usage
    exit 2
fi

# SemVer 形式チェック（vX.Y.Z または vX.Y.Z-pre）
if ! [[ "$VERSION" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[A-Za-z0-9.]+)?$ ]]; then
    echo "[error] $VERSION は SemVer 形式（vX.Y.Z または vX.Y.Z-pre）でない" >&2
    exit 2
fi

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$REPO_ROOT"

# 1. git status クリーン確認
if [[ -n "$(git status --porcelain)" ]]; then
    echo "[error] git status が dirty。release tag は clean tree で切る必要がある" >&2
    git status --short >&2
    exit 2
fi

# 2. tag が既存でないか確認
if git rev-parse "$VERSION" >/dev/null 2>&1; then
    echo "[error] tag $VERSION は既存。force 上書きは禁止" >&2
    exit 2
fi

# 3. make qualify-release 強制実行（dry-run でない場合）
if [[ "$DRY_RUN" -eq 0 ]]; then
    echo "[info] make qualify-release を実行（L0–L5 + L7 + L9 + L10 全層）"
    if command -v make >/dev/null 2>&1; then
        # make が使える環境では qualify-release target を呼ぶ
        if ! make qualify-release; then
            echo "[error] make qualify-release が失敗。tag 切り中止" >&2
            exit 1
        fi
    else
        echo "[error] make コマンドが不在。release tag 切りには make qualify-release が必須" >&2
        exit 2
    fi
else
    echo "[info] --dry-run: qualify-release を skip（tag 切りは実行）"
fi

# 4. qualify report tar.zst 化（report ディレクトリが存在する場合のみ）
QUALIFY_REPORT_DIR="${REPO_ROOT}/tests/qualify-report/${VERSION}"
QUALIFY_REPORT_HASH="no-report"
if [[ -d "$QUALIFY_REPORT_DIR" ]]; then
    REPORT_TAR="${QUALIFY_REPORT_DIR}.tar.zst"
    if command -v zstd >/dev/null 2>&1; then
        tar -cf - -C "${QUALIFY_REPORT_DIR}/.." "$(basename "$QUALIFY_REPORT_DIR")" | zstd -19 -o "$REPORT_TAR"
        QUALIFY_REPORT_HASH="$(sha256sum "$REPORT_TAR" | cut -d' ' -f1)"
        echo "[info] qualify report archived: $REPORT_TAR (sha256=$QUALIFY_REPORT_HASH)"
    else
        # zstd 不在なら gzip にフォールバック（ADR-TEST-001 portable 制約と整合）
        REPORT_TAR="${QUALIFY_REPORT_DIR}.tar.gz"
        tar -czf "$REPORT_TAR" -C "${QUALIFY_REPORT_DIR}/.." "$(basename "$QUALIFY_REPORT_DIR")"
        QUALIFY_REPORT_HASH="$(sha256sum "$REPORT_TAR" | cut -d' ' -f1)"
        echo "[info] qualify report archived: $REPORT_TAR (sha256=$QUALIFY_REPORT_HASH, zstd 不在のため gzip)"
    fi
fi

# 5. git tag を切る（メッセージに qualify report の sha256 を埋め込む）
TAG_MSG="release ${VERSION}

qualify-report-sha256: ${QUALIFY_REPORT_HASH}
qualify-mode: $([[ "$DRY_RUN" -eq 1 ]] && echo "dry-run" || echo "full")
"
echo "[info] git tag -a $VERSION"
git tag -a "$VERSION" -m "$TAG_MSG"

echo "[done] tag $VERSION created. push は別操作（git push origin $VERSION）"
exit 0
