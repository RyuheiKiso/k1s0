#!/usr/bin/env bash
#
# Dev Container 起動後の初期化と toolchain 検証.
#
# 設計書: docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md
# IMP-DEV-DC-016: postCreate script (pre-commit / mise / make seed の初回実行)
# IMP-DEV-DC-017: time-to-first-commit 計測点 (postcreate-duration)
#
# 実行内容:
#   1. toolchain version 表示
#   2. tools/sparse/checkout-role.sh <role> --verify  : 役割と cone の整合チェック
#   3. tools/local-stack/up.sh --role <role>          : kind + 本番再現スタックを起動
#       - 既定: foreground 実行（時間はかかるが進捗が直接見える）
#       - K1S0_SKIP_LOCAL_STACK=1 でスキップ
#       - K1S0_LOCAL_STACK_BG=1 で背景起動 (.devcontainer/.log/local-stack-up.log に出力)

set -euo pipefail

START_TS=$(date +%s)

ROLE="${K1S0_DEV_ROLE:-docs-writer}"
REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"

echo "## [postCreate] role=${ROLE}"

echo
echo "## [postCreate] toolchain 検証"
toolcheck() {
    if command -v "$1" >/dev/null 2>&1; then
        printf '  %-12s %s\n' "$1:" "$($1 --version 2>&1 | head -1)"
    fi
}
toolcheck rustc
toolcheck cargo
toolcheck go
toolcheck dotnet
toolcheck node
toolcheck pnpm
toolcheck protoc
toolcheck buf
toolcheck kubectl
toolcheck helm
toolcheck kind
toolcheck istioctl
toolcheck argocd
toolcheck dapr
toolcheck temporal
toolcheck cosign
toolcheck syft
toolcheck drawio-export
toolcheck pandoc
toolcheck markdownlint-cli2
toolcheck textlint
toolcheck mmdc

if ! command -v dapr >/dev/null 2>&1; then
    echo "## [postCreate] Dapr CLI を導入"
    if curl -fsSL https://raw.githubusercontent.com/dapr/cli/master/install/install.sh | sudo /bin/bash 2>/dev/null; then
        :
    elif curl -fsSL https://raw.githubusercontent.com/dapr/cli/master/install/install.sh | /bin/bash 2>/dev/null; then
        :
    else
        echo "  [warn] dapr CLI 導入に失敗。手動で実行してください"
    fi
fi

echo
echo "## [postCreate] sparse-checkout 整合チェック"
if [[ -x "${REPO_ROOT}/tools/sparse/checkout-role.sh" ]]; then
    "${REPO_ROOT}/tools/sparse/checkout-role.sh" "${ROLE}" --verify || \
        echo "  [info] sparse-checkout の整合は要確認（手動で 'tools/sparse/checkout-role.sh ${ROLE}' を実行）"
fi

echo
echo "## [postCreate] local-stack 起動"
if [[ "${K1S0_SKIP_LOCAL_STACK:-0}" == "1" ]]; then
    echo "  K1S0_SKIP_LOCAL_STACK=1 のためスキップ"
elif [[ ! -x "${REPO_ROOT}/tools/local-stack/up.sh" ]]; then
    echo "  tools/local-stack/up.sh が見つからないためスキップ"
elif ! command -v docker >/dev/null 2>&1 || ! docker info >/dev/null 2>&1; then
    echo "  [warn] docker daemon に接続できないため local-stack 起動をスキップ"
elif [[ "${K1S0_LOCAL_STACK_BG:-0}" == "1" ]]; then
    LOG_DIR="${REPO_ROOT}/.devcontainer/.log"
    mkdir -p "${LOG_DIR}"
    LOG_FILE="${LOG_DIR}/local-stack-up.log"
    echo "  K1S0_LOCAL_STACK_BG=1: バックグラウンド起動（log: ${LOG_FILE}）"
    nohup "${REPO_ROOT}/tools/local-stack/up.sh" --role "${ROLE}" \
        >"${LOG_FILE}" 2>&1 &
    disown || true
    echo "  進捗確認: tail -f ${LOG_FILE}"
else
    echo "  foreground 実行（停止は Ctrl-C、背景化は K1S0_LOCAL_STACK_BG=1）"
    "${REPO_ROOT}/tools/local-stack/up.sh" --role "${ROLE}" \
        || echo "  [warn] local-stack 起動が失敗。tools/local-stack/status.sh で状態確認"
fi

END_TS=$(date +%s)
DURATION=$((END_TS - START_TS))

echo
echo "## [postCreate] 完了 (postcreate-duration=${DURATION}s, role=${ROLE})"
echo "  IMP-DEV-DC-017 計測点: postcreate-duration=${DURATION} 秒"
