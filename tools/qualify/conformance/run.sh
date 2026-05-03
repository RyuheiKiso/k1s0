#!/usr/bin/env bash
#
# tools/qualify/conformance/run.sh
#
# CNCF Conformance テスト（Sonobuoy）の冪等実行 orchestrator。
#
# 設計正典: ADR-TEST-003（CNCF Conformance を Sonobuoy + kind multi-node + Calico で月次実行）
# 関連: ADR-CNCF-001 / IMP-CI-CONF-001〜005
#
# Usage:
#   tools/qualify/conformance/run.sh                  # 全工程（cluster 起動 → 実行 → retrieve → cleanup）
#   tools/qualify/conformance/run.sh --skip-up        # 既存 cluster で実行（local-stack 起動済前提）
#   tools/qualify/conformance/run.sh --keep-cluster   # 実行後 cluster を削除しない
#
# 出力:
#   tests/.conformance/<YYYY-MM>/sonobuoy-results.tar.gz
#   tests/.conformance/<YYYY-MM>/summary.md
#
# 環境変数:
#   SONOBUOY_VERSION  Sonobuoy CLI のバージョン（既定 v0.57.3）
#   K1S0_KIND_NAME    kind cluster 名（既定 k1s0-local、tools/local-stack/up.sh と整合）
#
# 終了コード:
#   0 = 全 conformance test PASS / 1 = failed test あり / 2 = 引数エラー or 環境不備

set -euo pipefail

# ヘルプメッセージ
usage() {
    sed -n '2,25p' "$0" | sed 's/^# \{0,1\}//'
}

# 引数解析
SKIP_UP=0
KEEP_CLUSTER=0
for arg in "$@"; do
    case "$arg" in
        --skip-up) SKIP_UP=1 ;;
        --keep-cluster) KEEP_CLUSTER=1 ;;
        -h|--help) usage; exit 0 ;;
        *) echo "[error] unknown arg: $arg" >&2; usage; exit 2 ;;
    esac
done

# 必須 binary 確認
require_bin() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "[error] $1 not found in PATH（採用初期で devcontainer profile に追加）" >&2
        return 1
    fi
}
require_bin kubectl || exit 2
require_bin kind || exit 2

# Sonobuoy CLI が無ければ案内（自動 install しない、ADR-TEST-001 の portable 制約と整合）
if ! command -v sonobuoy >/dev/null 2>&1; then
    echo "[error] sonobuoy CLI not found"
    echo "  install: https://github.com/vmware-tanzu/sonobuoy/releases"
    echo "  例: curl -L https://github.com/vmware-tanzu/sonobuoy/releases/download/v0.57.3/sonobuoy_0.57.3_linux_amd64.tar.gz | tar xz && sudo mv sonobuoy /usr/local/bin/"
    exit 2
fi

# repo root を取得（cd-safe）
REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$REPO_ROOT"

# artifact 出力先（YYYY-MM）
YEAR_MONTH="$(date -u +%Y-%m)"
OUT_DIR="${REPO_ROOT}/tests/.conformance/${YEAR_MONTH}"
mkdir -p "$OUT_DIR"

# kind cluster 起動（--skip-up 指定時は既存 cluster を使う）
if [[ "$SKIP_UP" -eq 0 ]]; then
    echo "[info] kind cluster + Calico CNI を --role conformance で起動"
    "${REPO_ROOT}/tools/local-stack/up.sh" --role conformance
fi

# kind cluster の Ready を確認
echo "[info] cluster Ready 待機（最大 5 分）"
kubectl wait --for=condition=Ready node --all --timeout=300s

# Sonobuoy 実行（certified-conformance モード、500+ test を走らせる）
echo "[info] Sonobuoy certified-conformance 実行（所要 60〜120 分）"
sonobuoy run --mode certified-conformance --wait

# 結果 retrieve
echo "[info] 結果 retrieve"
TAR_PATH="${OUT_DIR}/sonobuoy-results.tar.gz"
sonobuoy retrieve > "$TAR_PATH"

# human readable summary 生成
echo "[info] summary 生成"
SUMMARY_PATH="${OUT_DIR}/summary.md"
{
    echo "# CNCF Conformance Results — ${YEAR_MONTH}"
    echo ""
    echo "## Sonobuoy Results"
    echo ""
    echo '```'
    sonobuoy results "$TAR_PATH"
    echo '```'
    echo ""
    echo "## Cluster Info"
    echo ""
    echo '```'
    kubectl version
    echo '```'
} > "$SUMMARY_PATH"

# failed test 数を取得して exit code に反映
FAILED=$(sonobuoy results "$TAR_PATH" --plugin e2e | grep -c "^failed: " || true)
if [[ "$FAILED" -gt 0 ]]; then
    echo "[error] ${FAILED} 件の test が failed"
    EXIT_CODE=1
else
    echo "[ok] 全 conformance test PASS"
    EXIT_CODE=0
fi

# cluster 削除（--keep-cluster で skip）
if [[ "$KEEP_CLUSTER" -eq 0 ]]; then
    echo "[info] sonobuoy cluster cleanup"
    sonobuoy delete --wait || true
    if [[ "$SKIP_UP" -eq 0 ]]; then
        echo "[info] kind cluster 削除"
        kind delete cluster --name "${K1S0_KIND_NAME:-k1s0-local}" || true
    fi
fi

echo "[done] artifact: ${TAR_PATH}"
echo "[done] summary: ${SUMMARY_PATH}"
exit "$EXIT_CODE"
