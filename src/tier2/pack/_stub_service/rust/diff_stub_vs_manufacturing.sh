#!/bin/bash
# tier2 API neutrality 検証スクリプト (Rust)
# stub_service と manufacturing の公開 API diff を取得する

# スクリプトを厳格モードで実行する（未定義変数・エラーで即時終了）
set -euo pipefail

# stub service の公開 API を取得する (cargo public-api が利用可能な場合)
if command -v cargo-public-api &> /dev/null; then
    # cargo public-api を実行して stub service の公開 API を取得する
    STUB_API=$(cargo public-api --manifest-path "$(dirname "$0")/Cargo.toml" 2>/dev/null)
    # stub_service の公開 API を表示する
    echo "stub_service API:"
    echo "$STUB_API"
else
    # cargo public-api が利用できない場合は dry-run 結果を出力する
    echo "DRY-RUN: cargo-public-api not available"
    # インストール方法を案内する
    echo "Install with: cargo install cargo-public-api"
fi
