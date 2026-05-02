#!/usr/bin/env bash
# =============================================================================
# ops/dr/scripts/restore-minio-from-archive.sh
#
# 設計: ops/dr/scenarios/minio-tenant-restore.md の手順を 1 コマンド化
# 関連 Runbook: RB-BKP-001
#
# Usage:
#   ops/dr/scripts/restore-minio-from-archive.sh \
#     --bucket k1s0-postgres-backup \
#     [--archive-alias minio-archive] \
#     [--target-alias minio] \
#     [--parallel 30] \
#     [--dry-run]
# =============================================================================
set -euo pipefail

BUCKET=""
ARCHIVE_ALIAS="minio-archive"
TARGET_ALIAS="minio"
PARALLEL=30
DRY_RUN=0

usage() {
    sed -n '3,15p' "$0" | sed 's/^# \{0,1\}//'
    exit 2
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --bucket) BUCKET="$2"; shift 2 ;;
        --archive-alias) ARCHIVE_ALIAS="$2"; shift 2 ;;
        --target-alias) TARGET_ALIAS="$2"; shift 2 ;;
        --parallel) PARALLEL="$2"; shift 2 ;;
        --dry-run) DRY_RUN=1; shift ;;
        -h|--help) usage ;;
        *) echo "[error] 未知: $1"; usage ;;
    esac
done

if [[ -z "${BUCKET}" ]]; then
    echo "[error] --bucket は必須" >&2
    usage
fi

# 必須ツール
command -v mc >/dev/null || { echo "[error] mc CLI 不在" >&2; exit 2; }

if [[ "${DRY_RUN}" == "1" ]]; then
    echo "[dry-run] mc mirror --parallel ${PARALLEL} --overwrite \\"
    echo "  ${ARCHIVE_ALIAS}/${BUCKET}-archive/ ${TARGET_ALIAS}/${BUCKET}/"
    exit 0
fi

# archive 存在確認
echo "[info] archive 存在確認"
mc ls "${ARCHIVE_ALIAS}/${BUCKET}-archive/" | tail -5

# 並列 mirror
echo "[info] mirror 開始 (parallel=${PARALLEL})"
mc mirror --parallel "${PARALLEL}" --overwrite \
    "${ARCHIVE_ALIAS}/${BUCKET}-archive/" \
    "${TARGET_ALIAS}/${BUCKET}/"

# 完了確認
echo "[info] target bucket の オブジェクト数確認"
mc ls --recursive "${TARGET_ALIAS}/${BUCKET}/" | wc -l
echo "[info] archive bucket の オブジェクト数確認"
mc ls --recursive "${ARCHIVE_ALIAS}/${BUCKET}-archive/" | wc -l

echo "[info] 復旧完了。tier1 facade / CNPG が新 bucket を再認識するまで rolling restart を検討"
