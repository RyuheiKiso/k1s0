#!/usr/bin/env bash
#
# tests/golden/diff-tool/compare-outputs.sh
#
# k1s0-scaffold が生成する <ServiceType> の出力を期待値（expected.tar.gz）と比較する。
# 差分があれば exit 1（CI が PR を block する）。
#
# 使い方:
#   tests/golden/diff-tool/compare-outputs.sh tier2-go-service
#   tests/golden/diff-tool/compare-outputs.sh tier3-web
#
# 設計正典: docs/05_実装/00_ディレクトリ設計/70_共通資産/02_tests配置.md

set -euo pipefail

# 引数チェック（ServiceType 名は src/platform/cli の SupportedTypes と整合する 4 種）
if [[ $# -ne 1 ]]; then
    echo "usage: $0 <tier2-go-service|tier2-dotnet-service|tier3-bff|tier3-web>" >&2
    exit 2
fi

SCAFFOLD_NAME="$1"

# リポジトリルートを git で解決
REPO_ROOT="$(git rev-parse --show-toplevel)"
EXPECTED_TGZ="${REPO_ROOT}/tests/golden/scaffold-outputs/${SCAFFOLD_NAME}/expected.tar.gz"

# expected が存在しない場合は明示的にスキップ（採用初期 で各 ServiceType の expected を埋める）
if [[ ! -f "${EXPECTED_TGZ}" ]]; then
    echo "[skip] expected.tar.gz が未配置: ${EXPECTED_TGZ}" >&2
    echo "[skip] 採用初期 で k1s0-scaffold の出力を tar.gz 化して配置すること" >&2
    exit 0
fi

# 一時ディレクトリで scaffold を実行し、expected を展開して diff
WORK="$(mktemp -d)"
trap 'rm -rf "${WORK}"' EXIT

mkdir -p "${WORK}/actual" "${WORK}/expected"

# k1s0-scaffold は src/platform/cli/cmd/k1s0-scaffold で go run できる前提（採用初期 で binary 化）
cd "${REPO_ROOT}/src/platform/cli"
go run ./cmd/k1s0-scaffold "${SCAFFOLD_NAME}" --name golden-fixture --owner k1s0-test --out "${WORK}/actual"

# 期待値を展開
tar -xzf "${EXPECTED_TGZ}" -C "${WORK}/expected/"

# 再帰 diff（差分があれば exit 1）
diff -r "${WORK}/actual" "${WORK}/expected/${SCAFFOLD_NAME}"
