#!/usr/bin/env bash
# =============================================================================
# tools/codegen/scaffold/regenerate-golden.sh
#
# Scaffold CLI が生成する全 ServiceType の出力を `tests/golden/scaffold-outputs/
# <ServiceType>/expected.tar.gz` に再生成する補助スクリプト。
#
# 設計: docs/05_実装/20_コード生成設計/30_Scaffold_CLI/01_Scaffold_CLI設計.md
# 関連 ID:
#   IMP-CODEGEN-SCF-030〜037 (Scaffold CLI テンプレート群)
#   IMP-CODEGEN-GLD-040〜047 (Golden snapshot)
#
# 使い方:
#   tools/codegen/scaffold/regenerate-golden.sh           # 全 4 ServiceType
#   tools/codegen/scaffold/regenerate-golden.sh tier2-go-service  # 単体
#
# 何をするか:
#   1. src/platform/scaffold/ で k1s0-scaffold を release ビルド
#   2. 各 ServiceType に対し scaffold new を実行（決定的引数で）
#   3. 結果を tar.gz 化（mtime/owner 固定で再現性担保）
#   4. tests/golden/scaffold-outputs/<ServiceType>/expected.tar.gz に上書き
#
# 実行タイミング:
#   - skeleton/ 配下のテンプレート (.hbs) を意図的に変更した PR で
#   - tests/golden/diff-tool/compare-outputs.sh が FAIL した時、変更が
#     意図通りであることを確認した上で
#
# 規約:
#   本スクリプト実行後は git diff で expected.tar.gz が変わっていることを
#   確認し、PR 本文に「なぜ scaffold 出力が変わったか」を明記する
#   （CODEOWNERS で SRE + Security の二重承認が要求される）。
# =============================================================================
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || (cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd))"
SCAFFOLD_BIN="${REPO_ROOT}/src/platform/scaffold/target/release/k1s0-scaffold"
GOLDEN_DIR="${REPO_ROOT}/tests/golden/scaffold-outputs"

# golden test の決定的入力。compare-outputs.sh と必ず一致させる。
GOLDEN_NAME="golden-fixture"
GOLDEN_OWNER="@k1s0/test"
GOLDEN_DESC="Golden fixture sample (auto-regenerated)"
GOLDEN_NS="K1s0.Tier2.GoldenFixture"
GOLDEN_MTIME="2026-01-01 00:00:00 UTC"

# 対象 ServiceType（複数指定なければ全 4 件）
DEFAULT_TARGETS=(tier2-go-service tier2-dotnet-service tier3-bff tier3-web)
TARGETS=("$@")
if [[ "${#TARGETS[@]}" -eq 0 ]]; then
    TARGETS=("${DEFAULT_TARGETS[@]}")
fi

# k1s0-scaffold の release バイナリを最新化
echo "[info] k1s0-scaffold release ビルド"
(cd "${REPO_ROOT}/src/platform/scaffold" && cargo build --release --quiet)

if [[ ! -x "${SCAFFOLD_BIN}" ]]; then
    echo "[error] release ビルド後も binary が見つからない: ${SCAFFOLD_BIN}" >&2
    exit 2
fi

# ServiceType 別に snapshot を再生成
for tmpl in "${TARGETS[@]}"; do
    out_tgz="${GOLDEN_DIR}/${tmpl}/expected.tar.gz"
    if [[ ! -d "${GOLDEN_DIR}/${tmpl}" ]]; then
        echo "[error] golden output 配置先がない: ${GOLDEN_DIR}/${tmpl}" >&2
        exit 2
    fi

    workdir="$(mktemp -d)"
    trap 'rm -rf "${workdir}"' EXIT

    # ${SCAFFOLD_NAME}/${GOLDEN_NAME}/<files> 階層を作るため subdir を一段噛ます
    mkdir -p "${workdir}/${tmpl}"

    # template ごとに必要な追加引数
    extra_args=()
    case "${tmpl}" in
        tier2-dotnet-service)
            extra_args+=(--namespace "${GOLDEN_NS}")
            ;;
    esac

    echo "[info] ${tmpl}: scaffold 生成"
    "${SCAFFOLD_BIN}" new "${tmpl}" \
        --name "${GOLDEN_NAME}" \
        --owner "${GOLDEN_OWNER}" \
        --description "${GOLDEN_DESC}" \
        --out "${workdir}/${tmpl}" \
        "${extra_args[@]}"

    echo "[info] ${tmpl}: tar.gz 化"
    # 決定的アーカイブを作る: ファイル順序固定 / mtime 固定 / owner 0
    tar --sort=name \
        --mtime="${GOLDEN_MTIME}" \
        --owner=0 --group=0 --numeric-owner \
        -czf "${out_tgz}" \
        -C "${workdir}" "${tmpl}"

    rm -rf "${workdir}"
    trap - EXIT

    size="$(stat -c '%s' "${out_tgz}")"
    echo "[ok]   ${tmpl}: ${out_tgz} (${size} bytes)"
done

echo ""
echo "[done] 全 ${#TARGETS[@]} ServiceType の golden snapshot を再生成"
echo "[hint] git diff tests/golden/scaffold-outputs/ で変更内容を確認"
echo "[hint] 続いて tests/golden/diff-tool/compare-outputs.sh <ServiceType> で検証"
