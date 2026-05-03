#!/usr/bin/env bash
#
# tools/qualify/owner-e2e/update-results.sh — owner-e2e-results.md の entry 追記
#
# 設計正典:
#   ADR-TEST-011（release tag ゲート: results.md template）
#   docs/05_実装/30_CI_CD設計/35_e2e_test_design/40_release_tag_gate/03_owner_e2e_results_template.md
#
# Usage:
#   tools/qualify/owner-e2e/update-results.sh <YYYY-MM-DD>
#
# 処理:
#   1. tests/.owner-e2e/<日付>/sha256.txt を読み込み（archive.sh が生成）
#   2. tests/.owner-e2e/<日付>/{部位}/result.json を集計し PASS / FAIL 数を算出
#   3. owner-e2e-results.md の ## 月次サマリ 直下に新 entry を挿入
#   4. git add (commit は手動)
#
# 終了コード:
#   0 = entry 追加成功 / 1 = artifact 不在 / 2 = 引数エラー

set -euo pipefail

if [[ $# -lt 1 ]]; then
    echo "[error] 引数必須: <YYYY-MM-DD>" >&2
    exit 2
fi
RUN_DATE="$1"
if ! [[ "$RUN_DATE" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]]; then
    echo "[error] 日付形式不正: $RUN_DATE" >&2
    exit 2
fi

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
ARTIFACT_DIR="${REPO_ROOT}/tests/.owner-e2e/${RUN_DATE}"
RESULTS_MD="${REPO_ROOT}/docs/40_運用ライフサイクル/owner-e2e-results.md"

if [[ ! -d "$ARTIFACT_DIR" ]]; then
    echo "[error] artifact ディレクトリ不在: $ARTIFACT_DIR" >&2
    exit 1
fi

# sha256.txt から sha256 を読み込み
SHA256_FILE="${ARTIFACT_DIR}/sha256.txt"
if [[ ! -f "$SHA256_FILE" ]]; then
    echo "[error] sha256.txt 不在: $SHA256_FILE （archive.sh を先に実行）" >&2
    exit 1
fi
SHA256="$(cat "$SHA256_FILE")"

# 各部位の PASS / FAIL を集計（result.json は go test -json 形式）
count_pass_fail() {
    # 引数 1: 部位ディレクトリ
    local part_dir="$1"
    local result_json="${ARTIFACT_DIR}/${part_dir}/result.json"
    if [[ ! -f "$result_json" ]]; then
        echo "?/?"
        return
    fi
    # go test -json で各 test の Action: pass / fail を集計
    local pass_count fail_count total
    pass_count="$(grep -c '"Action":"pass"' "$result_json" 2>/dev/null || echo 0)"
    fail_count="$(grep -c '"Action":"fail"' "$result_json" 2>/dev/null || echo 0)"
    total=$((pass_count + fail_count))
    echo "${pass_count}/${total}"
}

PLATFORM_RESULT="$(count_pass_fail platform)"
OBSERVABILITY_RESULT="$(count_pass_fail observability)"
SECURITY_RESULT="$(count_pass_fail security)"
HA_DR_RESULT="$(count_pass_fail ha-dr)"
UPGRADE_RESULT="$(count_pass_fail upgrade)"
SDK_RESULT="$(count_pass_fail sdk-roundtrip)"
TIER3_WEB_RESULT="$(count_pass_fail tier3-web)"
PERF_RESULT="$(count_pass_fail perf)"

# 全部位 PASS 判定（FAIL があれば 1 件以上）
TOTAL_FAIL=0
for r in "$PLATFORM_RESULT" "$OBSERVABILITY_RESULT" "$SECURITY_RESULT" "$HA_DR_RESULT" \
    "$UPGRADE_RESULT" "$SDK_RESULT" "$TIER3_WEB_RESULT" "$PERF_RESULT"; do
    pass_part="${r%%/*}"
    total_part="${r##*/}"
    if [[ "$pass_part" != "$total_part" ]]; then
        TOTAL_FAIL=$((TOTAL_FAIL + 1))
    fi
done

if [[ "$TOTAL_FAIL" -eq 0 ]]; then
    JUDGEMENT="PASS"
else
    JUDGEMENT="FAIL（${TOTAL_FAIL} 部位で失敗）"
fi

# artifact path（zst / gz fallback 判定）
if [[ -f "${ARTIFACT_DIR}/full-result.tar.zst" ]]; then
    ARTIFACT_PATH="tests/.owner-e2e/${RUN_DATE}/full-result.tar.zst"
else
    ARTIFACT_PATH="tests/.owner-e2e/${RUN_DATE}/full-result.tar.gz"
fi

# entry 本文を組み立て
ENTRY=$(cat <<EOF
### ${RUN_DATE}

- 実走者: $(git config user.name 2>/dev/null || echo "unknown")
- 判定: ${JUDGEMENT}
- 各部位 PASS 数:
  - platform: ${PLATFORM_RESULT}
  - observability: ${OBSERVABILITY_RESULT}
  - security: ${SECURITY_RESULT}
  - ha-dr: ${HA_DR_RESULT}
  - upgrade: ${UPGRADE_RESULT}
  - sdk-roundtrip: ${SDK_RESULT}
  - tier3-web: ${TIER3_WEB_RESULT}
  - perf: ${PERF_RESULT}
- artifact sha256: ${SHA256}
- artifact path: ${ARTIFACT_PATH}
- 実走環境:
  - host: $(uname -srm 2>/dev/null || echo unknown)
  - kubectl: $(kubectl version --client -o yaml 2>/dev/null | grep gitVersion: | head -1 | awk '{print $2}' || echo unknown)
- 所要時間: （手動記入）
- 失敗詳細: $([[ "$JUDGEMENT" == "PASS" ]] && echo "なし" || echo "（手動で部位ごと記入）")

EOF
)

# 既存 owner-e2e-results.md があれば「## 月次サマリ」直下に挿入、無ければ新規作成
if [[ ! -f "$RESULTS_MD" ]]; then
    echo "[error] $RESULTS_MD 不在。先に owner-e2e-results.md を生成してください" >&2
    exit 1
fi

# ## 月次サマリ 直後の空行を見つけて、その下に entry を挿入
TMP_MD="$(mktemp)"
awk -v entry="$ENTRY" '
    /^## 月次サマリ/ { print; insert=1; next }
    insert && /^$/ { print; print entry; insert=0; next }
    { print }
' "$RESULTS_MD" > "$TMP_MD"
mv "$TMP_MD" "$RESULTS_MD"

echo "[ok] owner-e2e-results.md に ${RUN_DATE} entry 追加（判定: ${JUDGEMENT}）"
echo "[hint] 所要時間 + 失敗詳細を手動で埋めてから git commit してください"
exit 0
