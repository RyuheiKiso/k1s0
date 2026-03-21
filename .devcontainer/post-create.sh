#!/bin/bash
# .devcontainer/post-create.sh
# NOTE: Flutter は devcontainer.json の features で既にインストール済み
# このスクリプトでは重複インストールしない

set -e

# Go ツール（バージョン固定で再現性を確保する）
# golangci-lint は ci.yaml の Install golangci-lint ステップと同一バージョンを使用する
go install golang.org/x/tools/cmd/goimports@v0.31.0
go install github.com/golangci/golangci-lint/cmd/golangci-lint@v1.64.8
go install google.golang.org/protobuf/cmd/protoc-gen-go@v1.36.3
go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@v1.5.1

# oapi-codegen (OpenAPI Go コード生成)
go install github.com/oapi-codegen/oapi-codegen/v2/cmd/oapi-codegen@v2.4.1

# Rust コンポーネント
rustup component add clippy rustfmt

# sqlx-cli（データベースマイグレーション管理ツール）
# postgres feature のみビルドして依存を最小化する
if ! command -v sqlx &>/dev/null; then
    cargo install sqlx-cli --no-default-features --features postgres --locked
fi

# k1s0 CLI（プロジェクト管理・コード生成ツール）
# リポジトリルートから直接インストールする
if ! command -v k1s0 &>/dev/null; then
    cargo install --path CLI/crates/k1s0-cli --locked
fi

# protobuf コンパイラ
sudo apt-get update && sudo apt-get install -y protobuf-compiler

# buf (Protocol Buffers ツール)
BUF_VERSION="1.47.2"
if ! command -v buf &>/dev/null || [[ "$(buf --version 2>/dev/null)" != "${BUF_VERSION}" ]]; then
    curl -sSL "https://github.com/bufbuild/buf/releases/download/v${BUF_VERSION}/buf-$(uname -s)-$(uname -m)" -o /usr/local/bin/buf
    chmod +x /usr/local/bin/buf
fi

# just（justfile タスクランナー）
if ! command -v just &>/dev/null; then
    curl -sSL https://just.systems/install.sh | bash -s -- --to /usr/local/bin
fi

# pnpm（TypeScript ワークスペース管理）
# corepack は Node.js に同梱されているため別途インストール不要
corepack enable pnpm

echo "Dev Container setup complete."
echo ""

# セットアップ完了後のツール確認
bash .devcontainer/health-check.sh
