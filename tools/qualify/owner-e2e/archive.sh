#!/usr/bin/env bash
#
# tools/qualify/owner-e2e/archive.sh — owner full e2e の artifact 集約
#
# 設計正典:
#   ADR-TEST-011（release tag ゲート: artifact sha256 検証）
#   docs/05_実装/30_CI_CD設計/35_e2e_test_design/40_release_tag_gate/02_artifact_保管.md
#
# Usage:
#   tools/qualify/owner-e2e/archive.sh <YYYY-MM-DD>
#
# 処理:
#   1. tests/.owner-e2e/<日付>/ 配下を tar.zst で集約 (zstd -19、不在時 gzip)
#   2. cluster-info.txt + dmesg.txt を生成 (lib/artifact.sh 経由)
#   3. full-result.tar.zst の sha256 を計算 → tests/.owner-e2e/<日付>/sha256.txt
#
# 終了コード:
#   0 = 集約成功 / 1 = artifact ディレクトリ不在 / 2 = 引数エラー

set -euo pipefail

# 引数 1: 実走日（YYYY-MM-DD）
if [[ $# -lt 1 ]]; then
    echo "[error] 引数必須: tools/qualify/owner-e2e/archive.sh <YYYY-MM-DD>" >&2
    exit 2
fi
RUN_DATE="$1"
if ! [[ "$RUN_DATE" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]]; then
    echo "[error] 日付形式不正 (YYYY-MM-DD): $RUN_DATE" >&2
    exit 2
fi

# repo root + lib/artifact.sh source
REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
# shellcheck source=../../e2e/lib/common.sh
source "${REPO_ROOT}/tools/e2e/lib/common.sh"
# shellcheck source=../../e2e/lib/artifact.sh
source "${REPO_ROOT}/tools/e2e/lib/artifact.sh"

# artifact ディレクトリ確認
ARTIFACT_DIR="${REPO_ROOT}/tests/.owner-e2e/${RUN_DATE}"
if [[ ! -d "$ARTIFACT_DIR" ]]; then
    echo "[error] artifact ディレクトリ不在: $ARTIFACT_DIR" >&2
    exit 1
fi

# Step 1: cluster-info + dmesg を artifact 化（既に make 経由で生成済の場合は上書き）
e2e_collect_cluster_info "$ARTIFACT_DIR"
e2e_collect_dmesg "$ARTIFACT_DIR"

# Step 2: full-result.tar.zst を生成 + sha256 計算
TAR_PATH="${ARTIFACT_DIR}/full-result.tar.zst"
SHA256="$(e2e_archive_artifacts "$ARTIFACT_DIR" "$TAR_PATH")"

# zstd 不在で gzip fallback された場合は path 確認
if [[ ! -f "$TAR_PATH" ]] && [[ -f "${ARTIFACT_DIR}/full-result.tar.gz" ]]; then
    TAR_PATH="${ARTIFACT_DIR}/full-result.tar.gz"
fi

# Step 3: sha256.txt に記録（cut.sh の検証ターゲット）
echo "$SHA256" > "${ARTIFACT_DIR}/sha256.txt"

e2e_log "archive 完了:"
e2e_log "  tar:    ${TAR_PATH}"
e2e_log "  sha256: ${SHA256}"
e2e_log "  記録:   ${ARTIFACT_DIR}/sha256.txt"
exit 0
