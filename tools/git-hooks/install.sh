#!/usr/bin/env bash
#
# tools/git-hooks/install.sh — pre-commit hook 群のインストール
#
# 役割:
#   1. pre-commit がインストールされているか確認、無ければ案内
#   2. `pre-commit install` を実行（commit-msg / pre-commit / pre-push の各 stage）
#   3. 初回検証として `pre-commit run --all-files` を任意で実行
#
# Usage:
#   tools/git-hooks/install.sh                # hook 配置のみ
#   tools/git-hooks/install.sh --run-all      # 配置 + 全ファイル走査
#   tools/git-hooks/install.sh --uninstall    # hook 解除

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "${REPO_ROOT}"

UNINSTALL=0
RUN_ALL=0

for arg in "$@"; do
    case "$arg" in
        --uninstall) UNINSTALL=1 ;;
        --run-all) RUN_ALL=1 ;;
        -h|--help)
            sed -n '3,15p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *)
            echo "[error] 未知の引数: $arg" >&2
            exit 1
            ;;
    esac
done

if ! command -v pre-commit >/dev/null 2>&1; then
    echo "[error] pre-commit が見つかりません。"
    echo "        導入: pip install pre-commit  または  pipx install pre-commit"
    echo "        ※ Dev Container の docs-writer / sdk-dev / full プロファイルでは postCreate.sh で導入される予定。"
    exit 1
fi

if [[ "${UNINSTALL}" == "1" ]]; then
    pre-commit uninstall --hook-type pre-commit --hook-type commit-msg --hook-type pre-push
    echo "[ok] hook を解除しました"
    exit 0
fi

if [[ ! -f "${REPO_ROOT}/.pre-commit-config.yaml" ]]; then
    echo "[error] .pre-commit-config.yaml が見つかりません: ${REPO_ROOT}"
    exit 1
fi

# pre-commit / commit-msg / pre-push の各 stage に hook を仕込む
pre-commit install --hook-type pre-commit
pre-commit install --hook-type commit-msg
pre-commit install --hook-type pre-push

echo "[ok] pre-commit hook を配置しました"

if [[ "${RUN_ALL}" == "1" ]]; then
    echo "[info] --run-all: 全ファイルを対象に hook を実行します（時間がかかる場合あり）"
    pre-commit run --all-files
fi
