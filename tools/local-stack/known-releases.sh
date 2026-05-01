#!/usr/bin/env bash
#
# tools/local-stack/known-releases.sh — ADR-POL-002 canonical helm release set printer
#
# 用途:
#   1. CI (drift-check) が現 cluster の helm list と比較する基準として使う。
#   2. Kyverno policy block-non-canonical-helm-releases.yaml の allow-list と
#      手動同期しているか PR で機械検証する基準。
#
# 出力: canonical release 名を 1 行 1 件で stdout に出力。
#
# 依存: tools/local-stack/{up.sh, lib/apply-layers.sh} の
#       `helm upgrade --install <release-name>` 行と、本スクリプトの output、
#       Kyverno policy の allow-list が三者整合する必要がある。
#
# 注意: ADR-POL-002 finishing で up.sh が 500 行制限を超え apply_* 関数を
# lib/apply-layers.sh に分離したため、本スクリプトも両ファイルを走査対象とする。

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
UP_SH="${REPO_ROOT}/tools/local-stack/up.sh"
APPLY_LAYERS="${REPO_ROOT}/tools/local-stack/lib/apply-layers.sh"

# up.sh + lib/apply-layers.sh から `helm upgrade --install <release-name> ...` を grep で抽出
grep -hE '^\s*helm upgrade --install\s+\S+' "${UP_SH}" "${APPLY_LAYERS}" 2>/dev/null \
    | sed -E 's/.*helm upgrade --install\s+([A-Za-z0-9_-]+).*/\1/' \
    | sort -u

# istioctl install で生成される helm release も canonical 扱い
echo "istio-base"
echo "istio-cni"
echo "istiod"
echo "ztunnel"
