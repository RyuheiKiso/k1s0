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

# protobuf コンパイラ
sudo apt-get update && sudo apt-get install -y protobuf-compiler

# buf (Protocol Buffers ツール)
BUF_VERSION="1.47.2"
curl -sSL "https://github.com/bufbuild/buf/releases/download/v${BUF_VERSION}/buf-$(uname -s)-$(uname -m)" -o /usr/local/bin/buf
chmod +x /usr/local/bin/buf

echo "Dev Container setup complete."
