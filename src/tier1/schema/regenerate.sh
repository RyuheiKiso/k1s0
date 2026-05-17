#!/usr/bin/env bash
# tier1 schema codegen 再生成スクリプト
# buf generate を実行して gen/{go,typescript} を更新する。
# proto を変更した場合はこのスクリプトを実行すること。
set -euo pipefail

# スクリプトのディレクトリに移動する
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# GOPATH/bin を PATH に追加して protoc-gen-go 等を参照可能にする
export PATH="$PATH:$(go env GOPATH)/bin"

# buf generate を実行して全モジュールの codegen を実行する
echo "Running buf generate..."
buf generate
echo "Done. Generated files:"
# 生成されたファイルを一覧表示する
find gen/ -type f | sort
