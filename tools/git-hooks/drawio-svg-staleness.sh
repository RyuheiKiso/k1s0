#!/usr/bin/env bash
#
# tools/git-hooks/drawio-svg-staleness.sh — drawio と対応 SVG の鮮度チェック
#
# pre-commit hook から `*.drawio` の変更ファイル群を引数で受け取り、以下を検証する:
#   1. 各 *.drawio に対応する *.svg が存在するか
#   2. *.drawio が *.svg より新しくなっていないか（stale）
#
# SVG の自動再生成は行わない（hook の決定論性確保のため）。
# 違反があれば exit 1、SVG を tools/_export_svg.py で再生成して再 commit するよう案内する。

set -euo pipefail

if [[ $# -eq 0 ]]; then
    exit 0
fi

violations=0
for f in "$@"; do
    if [[ ! -f "$f" ]]; then
        continue
    fi
    base="${f%.drawio}"
    svg="${base}.svg"
    if [[ ! -e "$svg" ]]; then
        echo "[error] $f に対応する SVG が無い: $svg"
        violations=$((violations + 1))
        continue
    fi
    # *.drawio が *.svg より新しい = SVG が stale
    if [[ "$f" -nt "$svg" ]]; then
        echo "[error] $f が $svg より新しい（SVG が stale）"
        violations=$((violations + 1))
    fi
done

if [[ ${violations} -gt 0 ]]; then
    echo
    echo "対処: 以下のいずれかを実施してください"
    echo "  - python3 tools/_export_svg.py で SVG を再生成し、両方を git add"
    echo "  - drawio CLI を Dev Container 内で利用可能にする（docs-writer プロファイル）"
    exit 1
fi

exit 0
