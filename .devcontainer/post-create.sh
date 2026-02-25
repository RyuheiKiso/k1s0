#!/bin/bash
# .devcontainer/post-create.sh

set -e

# Go ツール
go install golang.org/x/tools/cmd/goimports@latest
go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest
go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest

# oapi-codegen (OpenAPI Go コード生成)
go install github.com/oapi-codegen/oapi-codegen/v2/cmd/oapi-codegen@latest

# Rust コンポーネント
rustup component add clippy rustfmt

# Flutter SDK
FLUTTER_VERSION="3.24.0"
git clone https://github.com/flutter/flutter.git -b "${FLUTTER_VERSION}" --depth 1 /opt/flutter
export PATH="/opt/flutter/bin:$PATH"
echo 'export PATH="/opt/flutter/bin:$PATH"' >> ~/.bashrc
flutter precache --web
flutter config --no-analytics

# Pre-commit フック
pip install pre-commit
pre-commit install

# protobuf コンパイラ
sudo apt-get update && sudo apt-get install -y protobuf-compiler

# buf (Protocol Buffers ツール)
BUF_VERSION="1.47.2"
curl -sSL "https://github.com/bufbuild/buf/releases/download/v${BUF_VERSION}/buf-$(uname -s)-$(uname -m)" -o /usr/local/bin/buf
chmod +x /usr/local/bin/buf

echo "Dev Container setup complete."
