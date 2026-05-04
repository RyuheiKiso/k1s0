#!/usr/bin/env bash
#
# tools/release/cut.sh — release tag を切る唯一の正規入口
#
# 設計正典:
#   ADR-TEST-001（release tag 強制 3 重防御）
#   ADR-TEST-007（4 段フェーズ: PR / nightly / weekly / release tag）
#   ADR-TEST-011（owner full e2e の代替保証: release tag ゲート）
# 関連 ID: IMP-CI-TAG-001〜005、IMP-CI-E2E-014（本拡張）
#
# Usage:
#   tools/release/cut.sh <version>           # 例: tools/release/cut.sh v0.1.0
#   tools/release/cut.sh --dry-run <version> # qualify-release / owner full PASS 検証を skip
#
# 環境変数:
#   OWNER_E2E_FRESHNESS_DAYS  owner full PASS の鮮度閾値（既定 30、範囲 1〜90、ADR-TEST-011）
#
# 強制ステップ:
#   1. git status クリーン確認（dirty なら exit 2）
#   2. version が SemVer 形式か確認
#   3. tag が既存でないか確認
#   4. owner full PASS 鮮度検証（ADR-TEST-011、--dry-run で skip 可）
#   5. owner full sha256 抽出 + artifact 整合検証（ADR-TEST-011）
#   6. make qualify-release 強制実行（L0–L5 + L7 + L9 + L10 全層）
#   7. qualify report を tar.zst 化
#   8. tag メッセージに qualify-report-sha256 + owner-e2e-result-sha256 + 実走日を埋め込む
#   9. git tag -a <version>
#  10. push は手動（cut.sh は tag 作成までで止め、push は別操作）
#
# 設計理由:
#   ADR-TEST-001 で「release tag を切る行為そのものに qualify 強制を物理的に紐付ける」
#   を決定し、ADR-TEST-011 で「owner full e2e の CI 不可を release tag で代替保証」を追加。
#   git tag 直叩きを止めて本 wrapper のみを正規入口とすることで、CI なし環境でも
#   qualify / owner full PASS 走行なしの release tag が原理的に切れない構造にする。
#
# 終了コード:
#   0 = tag 成功 / 1 = qualify or owner-e2e PASS 失敗 / 2 = 引数 / 環境エラー

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

# 3. owner full PASS 鮮度検証 + sha256 抽出（ADR-TEST-011、IMP-CI-E2E-014）
# 鮮度閾値は OWNER_E2E_FRESHNESS_DAYS env で 1〜90 日を許容（既定 30）。
OWNER_E2E_FRESHNESS_DAYS="${OWNER_E2E_FRESHNESS_DAYS:-30}"
OWNER_E2E_RESULTS_MD="${REPO_ROOT}/docs/40_運用ライフサイクル/owner-e2e-results.md"
OWNER_E2E_SHA256="no-record"
OWNER_E2E_DATE="no-record"
if [[ "$DRY_RUN" -eq 0 ]]; then
    if [[ ! "$OWNER_E2E_FRESHNESS_DAYS" =~ ^[0-9]+$ ]] || \
       [[ "$OWNER_E2E_FRESHNESS_DAYS" -lt 1 ]] || \
       [[ "$OWNER_E2E_FRESHNESS_DAYS" -gt 90 ]]; then
        echo "[error] OWNER_E2E_FRESHNESS_DAYS=${OWNER_E2E_FRESHNESS_DAYS} は 1〜90 の範囲外" >&2
        exit 2
    fi
    if [[ ! -f "$OWNER_E2E_RESULTS_MD" ]]; then
        echo "[error] $OWNER_E2E_RESULTS_MD が不在。owner full PASS 記録なしで release tag は切れない" >&2
        echo "[hint]  owner full を実走（host OS WSL2 native shell から make e2e-owner-full）し、PASS 記録を追記する" >&2
        exit 1
    fi
    # 最新 ### YYYY-MM-DD entry の日付を抽出（既定の order: 上に最新 = ADR-TEST-011 §3）
    # set -o pipefail 下で grep no-match (exit 1) が script を silent kill するのを `|| true` で回避
    LATEST_ENTRY_DATE="$(grep -m1 '^### [0-9]\{4\}-[0-9]\{2\}-[0-9]\{2\}' "$OWNER_E2E_RESULTS_MD" 2>/dev/null | sed 's/^### //' || true)"
    if [[ -z "$LATEST_ENTRY_DATE" ]]; then
        echo "[error] $OWNER_E2E_RESULTS_MD に ### YYYY-MM-DD entry が存在しない" >&2
        echo "[hint]  owner full を実走（host OS WSL2 native shell から make e2e-owner-full）し、PASS entry を追記する" >&2
        exit 1
    fi
    # 鮮度判定（現在から N 日以内）
    DAYS_DIFF=$(( ($(date +%s) - $(date -d "$LATEST_ENTRY_DATE" +%s)) / 86400 ))
    if [[ "$DAYS_DIFF" -lt 0 ]]; then
        echo "[error] owner full PASS 日付 ($LATEST_ENTRY_DATE) が未来。owner-e2e-results.md の改ざんを疑う" >&2
        exit 1
    fi
    if [[ "$DAYS_DIFF" -gt "$OWNER_E2E_FRESHNESS_DAYS" ]]; then
        echo "[error] owner full PASS が ${DAYS_DIFF} 日前 ($LATEST_ENTRY_DATE)、閾値 ${OWNER_E2E_FRESHNESS_DAYS} 日を超過" >&2
        echo "[hint]  owner full を再実走 + 新 entry を owner-e2e-results.md に追記してから cut.sh を再実行" >&2
        exit 1
    fi
    # 最新 entry の判定が PASS であることを確認（ADR-TEST-011 §1 検証項目）
    LATEST_ENTRY_BLOCK="$(awk -v d="### $LATEST_ENTRY_DATE" '
        $0==d {found=1; print; next}
        /^### / {if (found) exit}
        found {print}
    ' "$OWNER_E2E_RESULTS_MD")"
    if ! echo "$LATEST_ENTRY_BLOCK" | grep -q '^- 判定: PASS'; then
        echo "[error] owner-e2e-results.md 最新 entry ($LATEST_ENTRY_DATE) が PASS でない" >&2
        echo "[hint]  失敗詳細を確認し、修正対応 → 再実走 → 新 PASS entry 追加を行う" >&2
        exit 1
    fi
    # artifact sha256 を抽出（ADR-TEST-011 §1 step 5、IMP-CI-E2E-014）
    # 同じく pipefail 対策で `|| true` を付ける
    OWNER_E2E_SHA256="$(echo "$LATEST_ENTRY_BLOCK" | grep -m1 '^- artifact sha256:' 2>/dev/null | sed 's/^- artifact sha256: //' | tr -d ' \t' || true)"
    if [[ -z "$OWNER_E2E_SHA256" ]]; then
        echo "[error] owner-e2e-results.md 最新 entry に 'artifact sha256:' フィールド不在" >&2
        exit 1
    fi
    if ! [[ "$OWNER_E2E_SHA256" =~ ^[a-fA-F0-9]{64}$ ]]; then
        echo "[error] artifact sha256 形式不正（64 文字 HEX 期待、実: $OWNER_E2E_SHA256）" >&2
        exit 1
    fi
    # artifact ファイル存在 + sha256sum 整合確認
    OWNER_E2E_ARTIFACT="${REPO_ROOT}/tests/.owner-e2e/${LATEST_ENTRY_DATE}/full-result.tar.zst"
    if [[ ! -f "$OWNER_E2E_ARTIFACT" ]]; then
        # .tar.gz fallback（zstd 不在環境）
        OWNER_E2E_ARTIFACT="${REPO_ROOT}/tests/.owner-e2e/${LATEST_ENTRY_DATE}/full-result.tar.gz"
    fi
    if [[ ! -f "$OWNER_E2E_ARTIFACT" ]]; then
        echo "[error] owner full artifact 不在: tests/.owner-e2e/${LATEST_ENTRY_DATE}/full-result.tar.{zst,gz}" >&2
        echo "[hint]  git lfs pull で取得、または再実走で再生成" >&2
        exit 1
    fi
    ACTUAL_SHA256="$(sha256sum "$OWNER_E2E_ARTIFACT" | cut -d' ' -f1)"
    if [[ "$OWNER_E2E_SHA256" != "$ACTUAL_SHA256" ]]; then
        echo "[error] artifact sha256 不整合: 記録=$OWNER_E2E_SHA256 / 実=$ACTUAL_SHA256" >&2
        echo "[hint]  artifact 改ざん / 削除の可能性。再実走 + sha256 再計算" >&2
        exit 1
    fi
    OWNER_E2E_DATE="$LATEST_ENTRY_DATE"
    echo "[ok] owner full PASS 検証: ${LATEST_ENTRY_DATE} (${DAYS_DIFF} 日前) sha256=${OWNER_E2E_SHA256:0:16}..."
else
    echo "[info] --dry-run: owner full PASS 検証 skip"
fi

# 4. make qualify-release 強制実行（dry-run でない場合）
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

# 5. git tag を切る（メッセージに qualify-report + owner-e2e-result の sha256 を埋め込む）
# ADR-TEST-011 §1 step 8: tag メッセージに owner-e2e-result-sha256 + 実走日を含めて
# 採用検討者が `git show <tag>` で release 時点の owner full 検証証跡を一意に確認できる構造にする。
TAG_MSG="release ${VERSION}

qualify-report-sha256: ${QUALIFY_REPORT_HASH}
owner-e2e-result-sha256: ${OWNER_E2E_SHA256}
owner-e2e-result-date: ${OWNER_E2E_DATE}
qualify-mode: $([[ "$DRY_RUN" -eq 1 ]] && echo "dry-run" || echo "full")
"
echo "[info] git tag -a $VERSION"
git tag -a "$VERSION" -m "$TAG_MSG"

echo "[done] tag $VERSION created. push は別操作（git push origin $VERSION）"
exit 0
