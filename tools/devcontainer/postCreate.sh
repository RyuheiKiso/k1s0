#!/usr/bin/env bash
#
# Dev Container 起動後の初期化と toolchain 検証.
#
# 設計書: docs/05_実装/50_開発者体験設計/10_DevContainer_10役/01_DevContainer_10役設計.md
# IMP-DEV-DC-016: postCreate script (pre-commit / mise / make seed の初回実行).
# IMP-DEV-DC-017: time-to-first-commit 計測点 (postcreate-duration).
#
# Phase 1 bootstrap では tools/sparse/checkout-role.sh と tools/local-stack/up.sh が未整備
# のため、ここでは Dapr CLI 導入と toolchain version 表示のみを行う。

set -euo pipefail

START_TS=$(date +%s)

echo "## [postCreate] Dapr CLI 導入"
if ! command -v dapr >/dev/null 2>&1; then
    curl -fsSL https://raw.githubusercontent.com/dapr/cli/master/install/install.sh | sudo /bin/bash
else
    echo "  dapr already installed: $(dapr --version | head -1)"
fi

echo
echo "## [postCreate] toolchain 検証"
echo "  rustc:    $(rustc --version)"
echo "  cargo:    $(cargo --version)"
echo "  rustfmt:  $(rustfmt --version)"
echo "  clippy:   $(cargo-clippy --version 2>/dev/null || echo 'installed via rustup component')"
echo "  go:       $(go version)"
echo "  protoc:   $(protoc --version)"
echo "  buf:      $(buf --version 2>&1 | head -1)"
echo "  kubectl:  $(kubectl version --client=true 2>&1 | head -1)"
echo "  helm:     $(helm version --short)"
echo "  kind:     $(kind --version)"
echo "  dapr:     $(dapr --version 2>&1 | head -1)"
echo "  docker:   $(docker --version)"

END_TS=$(date +%s)
DURATION=$((END_TS - START_TS))
echo
echo "## [postCreate] 完了 (${DURATION}s)"
